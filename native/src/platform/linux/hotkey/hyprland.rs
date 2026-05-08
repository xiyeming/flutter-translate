#![allow(dead_code)]

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

#[allow(dead_code)]
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

        // Register binds via Hyprland IPC socket (.socket.sock)
        let mut action_map: HashMap<String, String> = HashMap::new();
        for s in &shortcuts {
            if !s.enabled { continue; }
            let bind = Self::to_hypr_bind(&s.key_combination);
            if bind.is_empty() { continue; }
            // Send "keyword bind MODS_KEY,exec,/bin/true" to register the hotkey
            // Note: Hyprland IPC format: "keyword bind SUPER_ALT,F,exec,/bin/true" (no "=", mods joined by _)
            let ipc_cmd = format!("keyword bind {},exec,/bin/true\n", bind);
            if Self::send_ipc(&ipc_cmd) {
                self.binds.push((bind.clone(), s.action.clone()));
                action_map.insert(bind, s.action.clone());
                tracing::info!("[hyprland] Registered bind: {}", s.key_combination);
            } else {
                tracing::warn!("[hyprland] Failed to register bind: {}", s.key_combination);
            }
        }

        if self.binds.is_empty() {
            return Err(HotkeyError::NoKeyboardDevices);
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let actions = Arc::new(action_map);
        tokio::task::spawn_blocking(move || {
            let mut stream = match UnixStream::connect(&socket2) {
                Ok(s) => {
                    tracing::info!("[hyprland] Connected to socket2: {}", socket2);
                    s
                }
                Err(e) => {
                    tracing::error!("[hyprland] Cannot connect to socket2 ({}): {}", socket2, e);
                    return;
                }
            };
            let mut buf = [0u8; 4096];
            while running.load(Ordering::SeqCst) {
                match stream.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let msg = String::from_utf8_lossy(&buf[..n]);
                        tracing::debug!("[hyprland] socket2 raw: {}", msg.trim());
                        if let Some(pos) = msg.find("activekeyv2>>") {
                            let event_str: String = msg[pos + 13..].trim().to_string();
                            tracing::info!("[hyprland] activekeyv2 event: {}", event_str);
                            for (bind, action) in actions.iter() {
                                if event_str.contains(bind.as_str()) {
                                    tracing::info!("[hyprland] Hotkey matched: {} -> {}", bind, action);
                                    let _ = event_tx.send(action.clone());
                                    break;
                                }
                            }
                        }
                    }
                    Ok(0) => {
                        tracing::warn!("[hyprland] socket2 returned 0 bytes, peer closed");
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Ok(_) => { std::thread::sleep(std::time::Duration::from_millis(50)); }
                    Err(e) => {
                        tracing::warn!("[hyprland] socket2 read error: {}", e);
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
        });

        tracing::info!("Hyprland listener started ({} shortcuts)", self.binds.len());
        Ok(())
    }

    pub fn unregister_all(&mut self) -> Result<(), HotkeyError> {
        self.running.store(false, Ordering::SeqCst);
        // Unbind all registered hotkeys via IPC
        for (bind, _) in &self.binds {
            let ipc_cmd = format!("keyword unbind {}\n", bind);
            let _ = Self::send_ipc(&ipc_cmd);
            tracing::info!("[hyprland] Unregistered bind: {}", bind);
        }
        self.binds.clear();
        Ok(())
    }
}
