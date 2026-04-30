use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ========== 翻译相关类型 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub provider_id: String,
    pub provider_name: String,
    pub source_text: String,
    pub translated_text: String,
    pub response_time_ms: u64,
    pub is_success: bool,
    pub error_message: Option<String>,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

// ========== 配置相关类型 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub api_key: Option<String>,
    pub api_url: Option<String>,
    pub model: String,
    pub auth_type: String,
    pub is_active: bool,
    pub sort_order: i32,
    pub system_prompt: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRule {
    pub id: String,
    pub provider_id: String,
    pub role_name: String,
    pub system_prompt: String,
    pub custom_rules: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub last_provider_id: String,
    pub last_compare_providers: Vec<String>,
    pub last_used: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub id: String,
    pub action: String,
    pub key_combination: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePref {
    pub code: String,
    pub display_name: String,
    pub usage_count: i32,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub id: String,
    pub theme: String,
    pub default_target_lang: String,
    pub auto_detect: bool,
    pub history_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub content: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ========== 系统相关类型 ==========

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesktopEnv {
    KdePlasma,
    Hyprland,
    Gnome,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f64,
    pub language: String,
    pub processing_time_ms: u64,
}


