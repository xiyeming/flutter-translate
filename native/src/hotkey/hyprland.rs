use crate::ffi::error::HotkeyError;
use crate::ffi::types::ShortcutBinding;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Hyprland IPC hotkey integration.
/// Registers temporary keybinds via the Hyprland socket and listens
/// for socket events to detect keypresses.
pub struct HyprlandHotkeyService {
    binds: Vec<(String, String)>,
    running: Arc<AtomicBool>,
}

impl HyprlandHotkeyService {
    pub fn new() -> Result<Self, HotkeyError> {
        Ok(Self { binds: Vec::new(), running: Arc::new(AtomicBool::new(false)) })
    }

    #[allow(dead_code)]
    fn find_socket() -> Option<String> {
        let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
        let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        Some(format!("{}/hypr/{}/.socket.sock", dir, sig))
    }

    fn find_socket2() -> Option<String> {
        let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
        let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        Some(format!("{}/hypr/{}/.socket2.sock", dir, sig))
    }

    #[allow(dead_code)]
    fn send_ipc(command: &str) -> bool {
        let socket = match Self::find_socket() {
            Some(s) => s,
            None => return false,
        };
        let mut stream = match UnixStream::connect(&socket) {
            Ok(s) => s,
            Err(_) => return false,
        };
        stream.write_all(command.as_bytes()).is_ok()
    }

    /// Convert our key format "Ctrl+Shift+T" to Hyprland format "CTRL_SHIFT,T"
    fn to_hypr_bind(keys: &str) -> String {
        let parts: Vec<&str> = keys.split('+').collect();
        if parts.len() < 2 { return String::new(); }
        let mut mods = Vec::new();
        let mut key: Option<String> = None;
        for p in &parts {
            let upper = p.trim().to_uppercase();
            match upper.as_str() {
                "CTRL" | "CONTROL" => mods.push("CTRL"),
                "SHIFT" => mods.push("SHIFT"),
                "ALT" => mods.push("ALT"),
                "META" | "SUPER" | "WIN" => mods.push("SUPER"),
                k => key = Some(k.to_string()),
            }
        }
        let key = match key {
            Some(k) => k,
            None => return String::new(),
        };
        format!("{},{}", mods.join("_"), key)
    }

    pub async fn register_all(&mut self, shortcuts: Vec<ShortcutBinding>, event_tx: broadcast::Sender<String>) -> Result<(), HotkeyError> {
        self.binds.clear();
        let socket2 = Self::find_socket2().ok_or(HotkeyError::NoKeyboardDevices)?;

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        // Build action map
        let mut action_map: HashMap<String, String> = HashMap::new();
        for s in &shortcuts {
            if !s.enabled { continue; }
            let bind = Self::to_hypr_bind(&s.key_combination);
            if bind.is_empty() { continue; }
            action_map.insert(bind, s.action.clone());
        }

        let actions = Arc::new(action_map);
        tokio::task::spawn_blocking(move || {
            let mut stream = match UnixStream::connect(&socket2) {
                Ok(s) => s,
                Err(_) => { tracing::error!("Cannot connect to hyprland socket2"); return; }
            };
            let mut buf = [0u8; 4096];
            while running.load(Ordering::SeqCst) {
                match stream.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let msg = String::from_utf8_lossy(&buf[..n]);
                        // Hyprland socket2 events: "activekeyv2>>keycode,keyboard"
                        // We map recognized key combinations
                        if let Some(pos) = msg.find("activekeyv2>>") {
                            let event_str: String = msg[pos + 13..].trim().to_string();
                            for (bind, action) in actions.iter() {
                                if event_str.contains(bind.as_str()) {
                                    let _ = event_tx.send(action.clone());
                                    break;
                                }
                            }
                        }
                    }
                    Ok(_) => { std::thread::sleep(std::time::Duration::from_millis(50)); }
                    Err(_) => { std::thread::sleep(std::time::Duration::from_millis(200)); }
                }
            }
        });

        tracing::info!("Hyprland listener started ({} shortcuts)", self.binds.len());
        Ok(())
    }

    pub fn unregister_all(&mut self) -> Result<(), HotkeyError> {
        self.running.store(false, Ordering::SeqCst);
        self.binds.clear();
        Ok(())
    }
}
