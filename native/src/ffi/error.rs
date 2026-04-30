use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranslateError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error from {provider}: {status} - {message}")]
    ApiError {
        provider: String,
        status: u16,
        message: String,
    },

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("No available providers")]
    NoAvailableProviders,

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Task join error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),

    #[error("Runtime error")]
    RuntimeError,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("API key missing for provider: {0}")]
    ApiKeyMissing(String),

    #[error("Request timeout")]
    Timeout,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("Secret service error: {0}")]
    SecretError(#[from] secret_service::Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Config not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
}

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("Portal error: {0}")]
    PortalError(#[from] anyhow::Error),

    #[error("Command execution failed: {0}")]
    CommandError(std::io::Error),

    #[error("Tesseract error: {0}")]
    TesseractError(String),

    #[error("User cancelled selection")]
    UserCancelled,

    #[error("Screenshot failed")]
    ScreenshotFailed,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("No text detected")]
    NoTextDetected,

    #[error("Permission denied")]
    PermissionDenied,
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("wl-clipboard error: {0}")]
    WlError(String),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Clipboard empty")]
    Empty,

    #[error("Channel send error")]
    ChannelError,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Write failed")]
    WriteFailed,
}

#[derive(Debug, Error)]
pub enum TrayError {
    #[error("D-Bus connection failed: {0}")]
    DbusError(#[from] zbus::Error),

    #[error("Menu registration failed: {0}")]
    MenuError(String),

    #[error("Watcher not available: {0}")]
    WatcherError(String),

    #[error("Init failed")]
    InitFailed,

    #[error("Icon not found")]
    IconNotFound,

    #[error("Notification failed")]
    NotificationFailed,
}

#[derive(Debug, Error)]
pub enum HotkeyError {
    #[error("D-Bus error: {0}")]
    DbusError(#[from] zbus::Error),

    #[error("evdev error: {0}")]
    EvdevError(String),

    #[error("Shortcut already registered: {0}")]
    AlreadyRegistered(String),

    #[error("No keyboard devices found")]
    NoKeyboardDevices,

    #[error("Unsupported desktop environment")]
    UnsupportedDesktop,
}
