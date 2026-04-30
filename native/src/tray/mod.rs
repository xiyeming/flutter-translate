pub mod indicator;
pub mod menu;

use crate::ffi::error::TrayError;
use self::indicator::TrayIndicator;
use self::menu::TrayMenu;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct TrayService {
    indicator: TrayIndicator,
    menu: TrayMenu,
    initialized: bool,
    action_tx: broadcast::Sender<String>,
}

impl TrayService {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel::<String>(16);
        Self {
            indicator: TrayIndicator::new(),
            menu: TrayMenu::new(),
            initialized: false,
            action_tx: tx,
        }
    }

    pub async fn init(&mut self) -> Result<(), TrayError> {
        self.indicator.register().await?;
        self.initialized = true;
        tracing::info!("Tray service initialized");
        Ok(())
    }

    pub fn show_notification(&self, title: &str, body: &str) -> Result<(), TrayError> {
        if !self.initialized {
            return Err(TrayError::InitFailed);
        }
        tracing::info!("Tray notification: {} - {}", title, body);
        Ok(())
    }

    pub async fn update_tooltip(&mut self, tooltip: &str) -> Result<(), TrayError> {
        self.indicator.update_tooltip(tooltip).await
    }
}

static TRAY_SERVICE: once_cell::sync::Lazy<tokio::sync::Mutex<TrayService>> =
    once_cell::sync::Lazy::new(|| {
        tokio::sync::Mutex::new(TrayService::new())
    });

pub async fn get_tray_service() -> tokio::sync::MutexGuard<'static, TrayService> {
    TRAY_SERVICE.lock().await
}

pub fn get_tray_service_sync() -> std::sync::MutexGuard<'static, TrayService> {
    static SYNC_SERVICE: once_cell::sync::Lazy<std::sync::Mutex<TrayService>> =
        once_cell::sync::Lazy::new(|| std::sync::Mutex::new(TrayService::new()));
    SYNC_SERVICE.lock().unwrap()
}
