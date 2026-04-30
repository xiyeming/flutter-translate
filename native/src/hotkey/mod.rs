pub mod kde;
pub mod hyprland;
pub mod evdev;

use crate::ffi::error::HotkeyError;
use crate::ffi::types::ShortcutBinding;
use crate::config::ConfigManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct HotkeyService {
    registered: Vec<ShortcutBinding>,
    event_tx: Option<broadcast::Sender<String>>,
    event_rx: Option<broadcast::Receiver<String>>,
    running: Arc<AtomicBool>,
}

impl HotkeyService {
    pub fn new() -> Self {
        Self {
            registered: Vec::new(),
            event_tx: None,
            event_rx: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn register_all(&mut self, shortcuts: Vec<ShortcutBinding>) -> Result<(), HotkeyError> {
        self.running.store(false, Ordering::SeqCst);
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let desktop_env = ConfigManager::detect_desktop_env();
        let (tx, rx) = broadcast::channel::<String>(32);
        self.event_tx = Some(tx.clone());
        self.event_rx = Some(rx);
        self.running = Arc::new(AtomicBool::new(true));

        // Hyprland: try evdev first (reliable with input group), IPC as fallback
        if desktop_env == crate::ffi::types::DesktopEnv::Hyprland {
            let mut evdev_service = evdev::EvdevHotkeyService::with_running(self.running.clone())?;
            if evdev_service.register_all(shortcuts.clone()).await.is_ok() {
                let event_tx = tx.clone();
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = evdev_service.listen_blocking(event_tx) {
                        tracing::warn!("Evdev listener failed: {}. Ensure input group: sudo usermod -aG input $USER", e);
                    }
                });
                self.registered = shortcuts;
                tracing::info!("Hyprland: evdev hotkeys registered");
                return Ok(());
            }
            // Fallback to Hyprland IPC
            let mut hl_service = hyprland::HyprlandHotkeyService::new()?;
            if hl_service.register_all(shortcuts.clone(), tx.clone()).await.is_ok() {
                self.registered = shortcuts;
                tracing::info!("Hyprland IPC hotkeys registered");
                return Ok(());
            }
            tracing::info!("Hyprland hotkeys failed");
            return Err(HotkeyError::NoKeyboardDevices);
        }

        // KDE: try KGlobalAccel
        if desktop_env == crate::ffi::types::DesktopEnv::KdePlasma {
            let mut kde_service = kde::KdeHotkeyService::new();
            if kde_service.register_all(shortcuts.clone()).await.is_ok() {
                self.registered = shortcuts;
                tracing::info!("KDE hotkeys registered successfully");
                return Ok(());
            }
            tracing::info!("KDE hotkey failed, falling back to evdev");
        }

        // Fallback: evdev
        let mut evdev_service = evdev::EvdevHotkeyService::with_running(self.running.clone())?;
        evdev_service.register_all(shortcuts.clone()).await?;

        let event_tx = tx;
        tokio::task::spawn_blocking(move || {
            if let Err(e) = evdev_service.listen_blocking(event_tx) {
                tracing::warn!(
                    "Evdev listener failed: {}. On Hyprland, ensure you are in the 'input' group: sudo usermod -aG input $USER",
                    e
                );
            }
        });

        self.registered = shortcuts;
        tracing::info!("Evdev hotkeys registered ({} shortcuts)", self.registered.len());
        Ok(())
    }

    pub fn unregister_all(&mut self) -> Result<(), HotkeyError> {
        self.running.store(false, Ordering::SeqCst);
        self.registered.clear();
        self.event_tx = None;
        self.event_rx = None;
        Ok(())
    }

    /// Poll for the next hotkey event. Returns None if no event available.
    pub fn poll_event(&mut self, _timeout_ms: u64) -> Option<String> {
        let rx = self.event_rx.as_mut()?;
        match rx.try_recv() {
            Ok(action) => Some(action),
            Err(broadcast::error::TryRecvError::Empty) => None,
            Err(_) => None,
        }
    }
}

// Single shared instance using tokio::sync::Mutex.
// Async access via .lock().await, sync access via .try_lock().
static HOTKEY_SERVICE: once_cell::sync::Lazy<Mutex<HotkeyService>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HotkeyService::new()));

pub async fn get_hotkey_service() -> tokio::sync::MutexGuard<'static, HotkeyService> {
    HOTKEY_SERVICE.lock().await
}

pub fn get_hotkey_service_sync() -> Option<tokio::sync::MutexGuard<'static, HotkeyService>> {
    HOTKEY_SERVICE.try_lock().ok()
}
