pub mod bridge;
pub mod types;
pub mod error;

pub use types::{
    TranslateRequest, TranslationResult, ProviderConfig, TranslationRule,
    ActiveSession, ShortcutBinding, LanguagePref, UserConfig, DesktopEnv,
    OcrResult, PromptTemplate,
};
pub use error::{
    TranslateError, ConfigError, OcrError, ClipboardError, TrayError, HotkeyError,
};
