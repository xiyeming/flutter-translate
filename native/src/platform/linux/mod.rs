use async_trait::async_trait;
use crate::ffi::types::{DesktopEnv, OcrResult, ShortcutBinding};
use crate::ffi::error::{ClipboardError, HotkeyError, OcrError, TrayError};
use crate::platform::PlatformBackend;

pub mod hotkey;
pub mod clipboard;
pub mod screenshot;
pub mod tray;

pub struct LinuxBackend {
    hotkey_service: tokio::sync::Mutex<hotkey::HotkeyService>,
    clipboard_service: Option<clipboard::ClipboardService>,
    ocr_service: tokio::sync::Mutex<crate::ocr::OcrService>,
    tray_service: tokio::sync::Mutex<tray::TrayService>,
}

impl LinuxBackend {
    pub fn new() -> Self {
        Self {
            hotkey_service: tokio::sync::Mutex::new(hotkey::HotkeyService::new()),
            clipboard_service: clipboard::ClipboardService::new().ok(),
            ocr_service: tokio::sync::Mutex::new(
                crate::ocr::OcrService::new().expect("Failed to initialize OCR service")
            ),
            tray_service: tokio::sync::Mutex::new(tray::TrayService::new()),
        }
    }
}

#[async_trait]
impl PlatformBackend for LinuxBackend {
    async fn register_hotkeys(&self, shortcuts: Vec<ShortcutBinding>) -> Result<(), HotkeyError> {
        let mut service = self.hotkey_service.lock().await;
        service.register_all(shortcuts).await
    }

    fn unregister_hotkeys(&self) -> Result<(), HotkeyError> {
        let mut service = self.hotkey_service.blocking_lock();
        service.unregister_all()
    }

    fn poll_hotkey_event(&self) -> Option<String> {
        match self.hotkey_service.try_lock() {
            Ok(mut service) => service.poll_event(0),
            Err(_) => None,
        }
    }

    fn get_clipboard_text(&self) -> Result<String, ClipboardError> {
        match &self.clipboard_service {
            Some(svc) => svc.get_text(),
            None => Err(ClipboardError::WlError("Clipboard service unavailable".into())),
        }
    }

    fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
        match &self.clipboard_service {
            Some(svc) => svc.set_text(text),
            None => Err(ClipboardError::WlError("Clipboard service unavailable".into())),
        }
    }

    async fn screenshot(&self) -> Result<Vec<u8>, OcrError> {
        let desktop_env = crate::config::ConfigManager::detect_desktop_env();
        screenshot::DesktopScreenshot::capture(&desktop_env)
    }

    async fn recognize(&self, image_data: Vec<u8>, lang: String) -> Result<OcrResult, OcrError> {
        let service = self.ocr_service.lock().await;
        service.recognize(&image_data, &lang).await
    }

    async fn init_tray(&self) -> Result<(), TrayError> {
        let mut service = self.tray_service.lock().await;
        service.init().await
    }

    fn show_tray_notification(&self, title: &str, body: &str) -> Result<(), TrayError> {
        let service = self.tray_service.blocking_lock();
        service.show_notification(title, body)
    }

    fn detect_desktop_env(&self) -> DesktopEnv {
        crate::config::ConfigManager::detect_desktop_env()
    }
}
