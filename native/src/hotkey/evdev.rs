use crate::ffi::error::HotkeyError;
use crate::ffi::types::ShortcutBinding;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

type ShortcutDef = (HashSet<u16>, u16, String);

pub struct EvdevHotkeyService {
    shortcuts: Vec<ShortcutDef>,
    running: Arc<AtomicBool>,
}

impl EvdevHotkeyService {
    pub fn new() -> Result<Self, HotkeyError> {
        Ok(Self {
            shortcuts: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn with_running(running: Arc<AtomicBool>) -> Result<Self, HotkeyError> {
        Ok(Self { shortcuts: Vec::new(), running })
    }

    fn parse_key_combination(keys: &str) -> Vec<u16> {
        keys.split('+')
            .filter_map(|k| {
                let key = k.trim().to_uppercase();
                match key.as_str() {
                    "CTRL" | "CONTROL" => Some(29),
                    "SHIFT" => Some(42),
                    "ALT" => Some(56),
                    "META" | "SUPER" | "WIN" => Some(125),
                    "F1" => Some(59), "F2" => Some(60), "F3" => Some(61),
                    "F4" => Some(62), "F5" => Some(63), "F6" => Some(64),
                    "F7" => Some(65), "F8" => Some(66), "F9" => Some(67),
                    "F10" => Some(68), "F11" => Some(87), "F12" => Some(88),
                    // Linux evdev keycodes (QWERTY positions, NOT alphabetical)
                    "A" => Some(30), "B" => Some(48), "C" => Some(46),
                    "D" => Some(32), "E" => Some(18), "F" => Some(33),
                    "G" => Some(34), "H" => Some(35), "I" => Some(23),
                    "J" => Some(36), "K" => Some(37), "L" => Some(38),
                    "M" => Some(50), "N" => Some(49), "O" => Some(24),
                    "P" => Some(25), "Q" => Some(16), "R" => Some(19),
                    "S" => Some(31), "T" => Some(20), "U" => Some(22),
                    "V" => Some(47), "W" => Some(17), "X" => Some(45),
                    "Y" => Some(21), "Z" => Some(44),
                    "0" => Some(11), "1" => Some(2), "2" => Some(3),
                    "3" => Some(4), "4" => Some(5), "5" => Some(6),
                    "6" => Some(7), "7" => Some(8), "8" => Some(9),
                    "9" => Some(10),
                    "SPACE" => Some(57),
                    "ESC" | "ESCAPE" => Some(1),
                    "TAB" => Some(15),
                    "ENTER" | "RETURN" => Some(28),
                    _ => None,
                }
            })
            .collect()
    }

    pub async fn register_all(&mut self, shortcuts: Vec<ShortcutBinding>) -> Result<(), HotkeyError> {
        self.shortcuts.clear();
        for binding in shortcuts {
            if !binding.enabled { continue; }
            let key_codes = Self::parse_key_combination(&binding.key_combination);
            if key_codes.len() < 2 { continue; }
            let modifiers: HashSet<u16> = key_codes[..key_codes.len()-1].iter().copied().collect();
            let trigger = key_codes[key_codes.len() - 1];
            self.shortcuts.push((modifiers, trigger, binding.action.clone()));
        }
        self.running.store(true, Ordering::SeqCst);
        tracing::info!("Evdev configured {} shortcuts", self.shortcuts.len());
        Ok(())
    }

    pub fn unregister_all(&mut self) -> Result<(), HotkeyError> {
        self.shortcuts.clear();
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("Evdev hotkey service stopped");
        Ok(())
    }

    pub fn listen_blocking(&self, tx: broadcast::Sender<String>) -> Result<(), HotkeyError> {
        let mut device_paths: Vec<std::path::PathBuf> = Vec::new();

        for (path, _) in evdev::enumerate() {
            device_paths.push(path);
        }

        if device_paths.is_empty() {
            tracing::warn!("No keyboard devices found for evdev");
            return Err(HotkeyError::NoKeyboardDevices);
        }

        tracing::info!("Evdev listener starting for {} candidate devices", device_paths.len());

        let running = self.running.clone();
        let held_keys = Arc::new(Mutex::new(HashSet::<u16>::new()));
        let shortcuts = self.shortcuts.clone();
        let last_trigger = Arc::new(Mutex::new(std::collections::HashMap::<String, Instant>::new()));
        let mut opened = 0u32;

        for path in device_paths {
            let mut device = match evdev::Device::open(&path) {
                Ok(d) if d.supported_keys().is_some() => d,
                _ => continue,
            };
            opened += 1;
            let tx = tx.clone();
            let running = running.clone();
            let held_keys = held_keys.clone();
            let shortcuts = shortcuts.clone();
            let last_trigger = last_trigger.clone();

            std::thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    match device.fetch_events() {
                        Ok(events) => {
                            for ev in events {
                                let code = ev.code() as u16;
                                if ev.value() == 1 {
                                    held_keys.lock().unwrap().insert(code);
                                    let keys = held_keys.lock().unwrap();
                                    for (mods, trigger, action) in &shortcuts {
                                        if code == *trigger && mods.iter().all(|m| keys.contains(m)) {
                                            let now = Instant::now();
                                            let mut lt = last_trigger.lock().unwrap();
                                            let last = lt.get(action).copied().unwrap_or(Instant::now() - Duration::from_secs(1));
                                            if now.duration_since(last).as_millis() < 400 { continue; }
                                            lt.insert(action.clone(), now);
                                            drop(lt);
                                            tracing::info!("Hotkey triggered: {}", action);
                                            let _ = tx.send(action.clone());
                                        }
                                    }
                                } else if ev.value() == 0 {
                                    held_keys.lock().unwrap().remove(&code);
                                }
                            }
                        }
                        Err(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                }
            });
        }

        if opened == 0 {
            return Err(HotkeyError::NoKeyboardDevices);
        }

        tracing::info!("Evdev listener started ({} keyboard devices opened)", opened);
        Ok(())
    }
}
