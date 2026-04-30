use flutter_rust_bridge::frb;
use crate::ffi::types::{TranslateRequest, TranslationResult, ProviderConfig, ActiveSession, ShortcutBinding, DesktopEnv, PromptTemplate};
use crate::ffi::error::{TranslateError, ConfigError, OcrError, ClipboardError, TrayError, HotkeyError};
use crate::config::ConfigManager;

/// 初始化所有服务（翻译引擎、配置管理等）
#[frb]
pub fn init_services() {
    crate::init_services();
}

// ========== 翻译服务 ==========

/// 使用配置覆盖进行翻译。
/// 如果存在保存的提供商配置，则使用其 api_key/base_url/model。
/// 否则，回退到环境变量和默认 URL。
#[frb]
pub async fn translate(
    text: String,
    source_lang: String,
    target_lang: String,
    provider_id: String,
    system_prompt_override: Option<String>,
) -> Result<TranslationResult, TranslateError> {
    let config = get_provider_config(&provider_id).await;
    let resolved = resolve_config(&provider_id, &config);
    let request = build_request(&text, &source_lang, &target_lang, &resolved, &system_prompt_override);
    let client = reqwest::Client::new();

    match provider_id.as_str() {
        "openai" | "deepseek" | "qwen" | "kimi" | "glm" => {
            translate_openai_compat(&client, &provider_id, &request, &config).await
        }
        "deepl" => translate_deepl(&client, &request, &config).await,
        "google" => translate_google(&client, &request, &config).await,
        "anthropic" => translate_anthropic(&client, &request, &config).await,
        "azure" => translate_azure(&client, &request, &config).await,
        "custom" => translate_custom(&client, &request, &config).await,
        _ => Err(TranslateError::ProviderNotFound(provider_id)),
    }
}

/// 比较多个提供商的翻译
#[frb]
pub async fn translate_compare(
    text: String,
    source_lang: String,
    target_lang: String,
    provider_ids: Vec<String>,
    system_prompt_override: Option<String>,
) -> Result<Vec<TranslationResult>, TranslateError> {
    if provider_ids.is_empty() {
        return Err(TranslateError::NoAvailableProviders);
    }

    let mut handles = Vec::new();
    for pid in provider_ids {
        let text = text.clone();
        let source_lang = source_lang.clone();
        let target_lang = target_lang.clone();
        let prompt_override = system_prompt_override.clone();
        handles.push(tokio::spawn(async move {
            translate(text, source_lang, target_lang, pid.clone(), prompt_override).await
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await.map_err(|e| TranslateError::TaskJoinError(e))? {
            Ok(result) => results.push(result),
            Err(e) => results.push(TranslationResult {
                provider_id: "unknown".to_string(),
                provider_name: "Unknown".to_string(),
                source_text: String::new(),
                translated_text: String::new(),
                response_time_ms: 0,
                is_success: false,
                error_message: Some(e.to_string()),
                prompt_tokens: 0,
                completion_tokens: 0,
            total_tokens: 0,
            }),
        }
    }
    Ok(results)
}

#[frb]
pub async fn detect_language(text: String) -> Result<String, TranslateError> {
    let _ = text;
    Ok("en".to_string())
}

// ========== 配置管理 ==========

#[frb]
pub async fn get_providers() -> Result<Vec<ProviderConfig>, ConfigError> {
    ConfigManager::get_all_providers().await
}

#[frb]
pub async fn save_provider(config: ProviderConfig) -> Result<(), ConfigError> {
    ConfigManager::save_provider(config).await
}

#[frb]
pub async fn delete_provider(id: String) -> Result<(), ConfigError> {
    ConfigManager::delete_provider(&id).await
}

/// 测试厂商连接结果
#[derive(Debug, Clone)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
}

/// 测试厂商连接，返回详细信息
#[frb]
pub async fn test_provider(provider_id: String) -> TestResult {
    let config = get_provider_config(&provider_id).await;
    let has_key = match &config {
        Some(c) => c.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false),
        None => get_env_api_key(&provider_id).is_some(),
    };
    if !has_key {
        return TestResult { success: false, message: "缺少 API Key，请在设置中填写 API Key".to_string() };
    }

    let resolved = resolve_config(&provider_id, &config);
    let request = build_request("Hello", "en", "zh", &resolved, &None);
    let client = reqwest::Client::new();

    let result = match provider_id.as_str() {
        "openai" | "deepseek" | "qwen" | "kimi" | "glm" => {
            translate_openai_compat(&client, &provider_id, &request, &config).await
        }
        "deepl" => translate_deepl(&client, &request, &config).await,
        "google" => translate_google(&client, &request, &config).await,
        "anthropic" => translate_anthropic(&client, &request, &config).await,
        "azure" => translate_azure(&client, &request, &config).await,
        "custom" => translate_custom(&client, &request, &config).await,
        _ => Err(TranslateError::ProviderNotFound(provider_id.clone())),
    };

    let success = match &result {
        Ok(r) => r.is_success,
        Err(_) => false,
    };
    let message = match &result {
        Ok(r) if r.is_success => "连接成功".to_string(),
        Ok(_) => "连接失败".to_string(),
        Err(e) => format!("连接失败: {}", e),
    };
    TestResult { success, message }
}

#[frb]
pub async fn get_active_session() -> Result<ActiveSession, ConfigError> {
    ConfigManager::get_active_session().await
}

#[frb]
pub async fn update_session(
    provider_id: Option<String>,
    compare_providers: Option<Vec<String>>,
) -> Result<(), ConfigError> {
    ConfigManager::update_session(provider_id, compare_providers).await
}

// ========== 提示词模板 ==========

#[frb]
pub async fn get_prompt_templates() -> Result<Vec<PromptTemplate>, ConfigError> {
    ConfigManager::get_all_prompt_templates().await
}

#[frb]
pub async fn save_prompt_template(tpl: PromptTemplate) -> Result<(), ConfigError> {
    ConfigManager::save_prompt_template(tpl).await
}

#[frb]
pub async fn delete_prompt_template(id: String) -> Result<(), ConfigError> {
    ConfigManager::delete_prompt_template(&id).await
}

// ========== 系统服务 ==========

#[frb]
pub fn detect_desktop_env() -> DesktopEnv {
    ConfigManager::detect_desktop_env()
}

#[frb]
pub async fn ocr_screenshot() -> Result<String, OcrError> {
    let ocr_service = crate::ocr::get_ocr_service().await;
    let image_data = ocr_service.screenshot().await?;
    let result = ocr_service.recognize(&image_data, "zh").await?;
    Ok(result.text)
}

#[frb]
pub async fn get_shortcuts() -> Result<Vec<ShortcutBinding>, ConfigError> {
    ConfigManager::get_all_shortcuts().await
}

#[frb]
pub async fn update_shortcut(binding: ShortcutBinding) -> Result<(), ConfigError> {
    ConfigManager::save_shortcut(binding).await
}

#[frb]
pub async fn register_hotkeys(shortcuts: Vec<ShortcutBinding>) -> Result<(), HotkeyError> {
    let mut hotkey_service = crate::hotkey::get_hotkey_service().await;
    hotkey_service.register_all(shortcuts).await
}

#[frb]
pub fn unregister_hotkeys() -> Result<(), HotkeyError> {
    if let Some(mut hotkey_service) = crate::hotkey::get_hotkey_service_sync() {
        hotkey_service.unregister_all()
    } else {
        Ok(())
    }
}

/// Poll for the next hotkey event. Returns None if no event available.
#[frb]
pub fn poll_hotkey_event() -> Option<String> {
    let mut hotkey_service = crate::hotkey::get_hotkey_service_sync()?;
    hotkey_service.poll_event(0)
}

// ========== 剪贴板服务 ==========

#[frb]
pub fn get_clipboard_text() -> Result<String, ClipboardError> {
    let clipboard = crate::clipboard::get_clipboard_service();
    clipboard.get_text()
}

#[frb]
pub fn set_clipboard_text(text: String) -> Result<(), ClipboardError> {
    let clipboard = crate::clipboard::get_clipboard_service();
    clipboard.set_text(text)
}

// ========== 托盘服务 ==========

#[frb]
pub async fn init_tray() -> Result<(), TrayError> {
    let mut tray = crate::tray::get_tray_service().await;
    tray.init().await
}

#[frb]
pub fn show_tray_notification(
    title: String,
    body: String,
) -> Result<(), TrayError> {
    let tray = crate::tray::get_tray_service_sync();
    tray.show_notification(&title, &body)
}

// ========== 内部辅助函数 ==========

struct ProviderResolvedConfig {
    api_key: String,
    base_url: String,
    model: String,
    system_prompt: Option<String>,
}

async fn get_provider_config(provider_id: &str) -> Option<ProviderConfig> {
    ConfigManager::get_all_providers().await.ok().and_then(|providers| {
        providers.into_iter().find(|p| p.id == provider_id && p.is_active)
    })
}

fn get_env_api_key(provider_id: &str) -> Option<String> {
    match provider_id {
        "openai" => std::env::var("OPENAI_API_KEY").ok(),
        "deepl" => std::env::var("DEEPL_API_KEY").ok(),
        "google" => std::env::var("GOOGLE_TRANSLATE_API_KEY").ok(),
        "qwen" => std::env::var("QWEN_API_KEY").ok().or_else(|| std::env::var("DASHSCOPE_API_KEY").ok()),
        "deepseek" => std::env::var("DEEPSEEK_API_KEY").ok(),
        "kimi" => std::env::var("KIMI_API_KEY").ok().or_else(|| std::env::var("MOONSHOT_API_KEY").ok()),
        "glm" => std::env::var("GLM_API_KEY").ok().or_else(|| std::env::var("ZHIPU_API_KEY").ok()),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
        "azure" => std::env::var("AZURE_OPENAI_API_KEY").ok(),
        "custom" => std::env::var("CUSTOM_API_KEY").ok(),
        _ => None,
    }
}

fn resolve_config(provider_id: &str, saved: &Option<ProviderConfig>) -> ProviderResolvedConfig {
    let default_urls = {
        let m: std::collections::HashMap<&str, &str> = [
            ("openai", "https://api.openai.com/v1"),
            ("deepseek", "https://api.deepseek.com/v1"),
            ("qwen", "https://dashscope.aliyuncs.com/compatible-mode/v1"),
            ("kimi", "https://api.moonshot.cn/v1"),
            ("glm", "https://open.bigmodel.cn/api/paas/v4"),
            ("anthropic", "https://api.anthropic.com/v1"),
            ("google", "https://translation.googleapis.com"),
            ("deepl", "https://api-free.deepl.com/v2"),
            ("azure", ""),
            ("custom", "https://api.openai.com/v1"),
        ].into_iter().collect();
        m
    };

    let default_models = {
        let m: std::collections::HashMap<&str, &str> = [
            ("openai", "gpt-4o-mini"),
            ("deepseek", "deepseek-chat"),
            ("qwen", "qwen-turbo"),
            ("kimi", "moonshot-v1-8k"),
            ("glm", "glm-4-plus"),
            ("anthropic", "claude-3-haiku-20240307"),
            ("google", "nmt"),
            ("deepl", "default"),
            ("azure", "gpt-4o-mini"),
            ("custom", "gpt-4o-mini"),
        ].into_iter().collect();
        m
    };

    match saved {
        Some(cfg) => ProviderResolvedConfig {
            api_key: cfg.api_key.clone().unwrap_or_else(|| get_env_api_key(provider_id).unwrap_or_default()),
            base_url: cfg.api_url.clone().unwrap_or_else(|| default_urls.get(provider_id).unwrap_or(&"").to_string()),
            model: if cfg.model.is_empty() { default_models.get(provider_id).unwrap_or(&"gpt-4o-mini").to_string() } else { cfg.model.clone() },
            system_prompt: cfg.system_prompt.clone(),
        },
        None => ProviderResolvedConfig {
            api_key: get_env_api_key(provider_id).unwrap_or_default(),
            base_url: default_urls.get(provider_id).unwrap_or(&"").to_string(),
            model: default_models.get(provider_id).unwrap_or(&"gpt-4o-mini").to_string(),
            system_prompt: None,
        },
    }
}

fn build_request(text: &str, source_lang: &str, target_lang: &str, resolved: &ProviderResolvedConfig, prompt_override: &Option<String>) -> TranslateRequest {
    let _source_desc = lang_name(source_lang);
    let target_desc = lang_name(target_lang);
    let default_prompt = format!(
        "You are a translation engine. Your ONLY job is to translate text. \\
Do NOT chat, explain, ask questions, or add commentary. \\
Translate the user's input into {}. \\
Output ONLY the translated text, with no extra words, quotes, or formatting.",
        target_desc
    );
    TranslateRequest {
        text: text.to_string(),
        source_lang: source_lang.to_string(),
        target_lang: target_lang.to_string(),
        model: Some(resolved.model.clone()),
        system_prompt: prompt_override.clone().or(resolved.system_prompt.clone()).or(Some(default_prompt)),
        temperature: Some(0.1),
    }
}

fn lang_name(code: &str) -> &str {
    match code {
        "auto" => "the target language",
        "zh" => "Chinese",
        "en" => "English",
        "ja" => "Japanese",
        "ko" => "Korean",
        "fr" => "French",
        "de" => "German",
        "es" => "Spanish",
        "ru" => "Russian",
        "pt" => "Portuguese",
        _ => code,
    }
}

fn build_base_url(raw: &str) -> String {
    let url = raw.trim_end_matches('/');
    url.to_string()
}

async fn translate_openai_compat(
    client: &reqwest::Client,
    provider_id: &str,
    request: &TranslateRequest,
    saved: &Option<ProviderConfig>,
) -> Result<TranslationResult, TranslateError> {
    let cfg = resolve_config(provider_id, saved);
    if cfg.api_key.is_empty() {
        return Err(TranslateError::ApiKeyMissing(provider_id.to_string()));
    }

    let path = if provider_id == "azure" { "/openai/deployments" } else { "/chat/completions" };
    let url = format!("{}/{}", build_base_url(&cfg.base_url), path.trim_start_matches('/'));

    let provider_names = {
        let m: std::collections::HashMap<&str, &str> = [
            ("openai", "OpenAI"),
            ("deepseek", "DeepSeek"),
            ("qwen", "Qwen"),
            ("kimi", "Kimi"),
            ("glm", "GLM"),
        ].into_iter().collect();
        m.get(provider_id).unwrap_or(&provider_id).to_string()
    };

    let default_prompt = format!(
        "You are a translation engine. Your ONLY job is to translate text. Do NOT chat, explain, ask questions, or add commentary. Translate the user's input from {} to {}. Output ONLY the translated text, with no extra words, quotes, or formatting.",
        request.source_lang, request.target_lang
    );
    let system_prompt = request.system_prompt.as_deref().unwrap_or(&default_prompt);

    let start = std::time::Instant::now();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": cfg.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": &request.text}
            ],
            "temperature": request.temperature.unwrap_or(0.3),
            "max_tokens": 4096,
        }))
        .send()
        .await
        .map_err(TranslateError::HttpError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TranslateError::ApiError {
            provider: provider_names.clone(),
            status: status.as_u16(),
            message: body,
        });
    }

    let json: serde_json::Value = response.json().await.map_err(|e| TranslateError::ApiError {
        provider: provider_names.clone(),
        status: 0,
        message: e.to_string(),
    })?;

    let translated_text = json["choices"][0]["message"]["content"]
        .as_str().unwrap_or("").trim().to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0);
    let total_tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0);

    Ok(TranslationResult {
        provider_id: provider_id.to_string(),
        provider_name: provider_names,
        source_text: request.text.clone(),
        translated_text,
        response_time_ms: start.elapsed().as_millis() as u64,
        is_success: true,
        error_message: None,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}

async fn translate_deepl(
    client: &reqwest::Client,
    request: &TranslateRequest,
    saved: &Option<ProviderConfig>,
) -> Result<TranslationResult, TranslateError> {
    let cfg = resolve_config("deepl", saved);
    if cfg.api_key.is_empty() {
        return Err(TranslateError::ApiKeyMissing("DeepL".to_string()));
    }

    let url = format!("{}/translate", build_base_url(&cfg.base_url));
    let start = std::time::Instant::now();

    let source_lang = if request.source_lang == "auto" { None } else { Some(request.source_lang.to_uppercase()) };
    let target_lang = request.target_lang.to_uppercase();

    let mut params = vec![
        ("text", request.text.clone()),
        ("target_lang", target_lang),
    ];
    if let Some(ref sl) = source_lang {
        params.push(("source_lang", sl.clone()));
    }

    let response = client
        .post(&url)
        .header("Authorization", format!("DeepL-Auth-Key {}", cfg.api_key))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(TranslateError::HttpError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TranslateError::ApiError { provider: "DeepL".to_string(), status: status.as_u16(), message: body });
    }

    let json: serde_json::Value = response.json().await.map_err(|e| TranslateError::ApiError {
        provider: "DeepL".to_string(), status: 0, message: e.to_string()
    })?;

    let translated_text = json["translations"][0]["text"].as_str().unwrap_or("").to_string();

    Ok(TranslationResult {
        provider_id: "deepl".to_string(),
        provider_name: "DeepL".to_string(),
        source_text: request.text.clone(),
        translated_text,
        response_time_ms: start.elapsed().as_millis() as u64,
        is_success: true,
        error_message: None,
        prompt_tokens: 0,
        completion_tokens: 0,
            total_tokens: 0,
    })
}


async fn translate_google(
    client: &reqwest::Client,
    request: &TranslateRequest,
    saved: &Option<ProviderConfig>,
) -> Result<TranslationResult, TranslateError> {
    let cfg = resolve_config("google", saved);
    if cfg.api_key.is_empty() {
        return Err(TranslateError::ApiKeyMissing("Google".to_string()));
    }

    let url = format!("{}/language/translate/v2?key={}", build_base_url(&cfg.base_url), cfg.api_key);
    let start = std::time::Instant::now();

    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "q": &request.text,
            "source": &request.source_lang,
            "target": &request.target_lang,
            "format": "text",
        }))
        .send()
        .await
        .map_err(TranslateError::HttpError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TranslateError::ApiError { provider: "Google".to_string(), status: status.as_u16(), message: body });
    }

    let json: serde_json::Value = response.json().await.map_err(|e| TranslateError::ApiError {
        provider: "Google".to_string(), status: 0, message: e.to_string()
    })?;

    let translated_text = json["data"]["translations"][0]["translatedText"].as_str().unwrap_or("").to_string();

    Ok(TranslationResult {
        provider_id: "google".to_string(),
        provider_name: "Google".to_string(),
        source_text: request.text.clone(),
        translated_text,
        response_time_ms: start.elapsed().as_millis() as u64,
        is_success: true,
        error_message: None,
        prompt_tokens: 0,
        completion_tokens: 0,
            total_tokens: 0,
    })
}


async fn translate_anthropic(
    client: &reqwest::Client,
    request: &TranslateRequest,
    saved: &Option<ProviderConfig>,
) -> Result<TranslationResult, TranslateError> {
    let cfg = resolve_config("anthropic", saved);
    if cfg.api_key.is_empty() {
        return Err(TranslateError::ApiKeyMissing("Anthropic".to_string()));
    }

    let url = format!("{}/messages", build_base_url(&cfg.base_url));
    let start = std::time::Instant::now();

    let default_prompt = format!(
        "You are a translation engine. Your ONLY job is to translate text. Do NOT chat, explain, ask questions, or add commentary. Translate the user's input from {} to {}. Output ONLY the translated text, with no extra words, quotes, or formatting.",
        request.source_lang, request.target_lang
    );
    let system_prompt = request.system_prompt.as_deref().unwrap_or(&default_prompt);

    let response = client
        .post(&url)
        .header("x-api-key", &cfg.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": cfg.model,
            "system": system_prompt,
            "messages": [{"role": "user", "content": &request.text}],
            "temperature": request.temperature.unwrap_or(0.3),
            "max_tokens": 4096,
        }))
        .send()
        .await
        .map_err(TranslateError::HttpError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TranslateError::ApiError { provider: "Anthropic".to_string(), status: status.as_u16(), message: body });
    }

    let json: serde_json::Value = response.json().await.map_err(|e| TranslateError::ApiError {
        provider: "Anthropic".to_string(), status: 0, message: e.to_string()
    })?;

    let translated_text = json["content"][0]["text"].as_str().unwrap_or("").trim().to_string();

    let prompt_tokens = json["usage"]["input_tokens"].as_u64().unwrap_or(0);
    let completion_tokens = json["usage"]["output_tokens"].as_u64().unwrap_or(0);

    Ok(TranslationResult {
        provider_id: "anthropic".to_string(),
        provider_name: "Anthropic".to_string(),
        source_text: request.text.clone(),
        translated_text,
        response_time_ms: start.elapsed().as_millis() as u64,
        is_success: true,
        error_message: None,
        prompt_tokens,
        completion_tokens,
            total_tokens: 0,
    })
}

async fn translate_azure(
    client: &reqwest::Client,
    request: &TranslateRequest,
    saved: &Option<ProviderConfig>,
) -> Result<TranslationResult, TranslateError> {
    let cfg = resolve_config("azure", saved);
    if cfg.api_key.is_empty() {
        return Err(TranslateError::ApiKeyMissing("Azure".to_string()));
    }
    if cfg.base_url.is_empty() {
        return Err(TranslateError::ApiKeyMissing("Azure endpoint".to_string()));
    }

    let deployment = &cfg.model;
    let api_version = "2024-08-01-preview";
    let url = format!("{}/openai/deployments/{}/chat/completions?api-version={}",
        build_base_url(&cfg.base_url), deployment, api_version);
    let start = std::time::Instant::now();

    let default_prompt = format!(
        "You are a translation engine. Your ONLY job is to translate text. Do NOT chat, explain, ask questions, or add commentary. Translate the user's input from {} to {}. Output ONLY the translated text, with no extra words, quotes, or formatting.",
        request.source_lang, request.target_lang
    );
    let system_prompt = request.system_prompt.as_deref().unwrap_or(&default_prompt);

    let response = client
        .post(&url)
        .header("api-key", &cfg.api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": &request.text}
            ],
            "temperature": request.temperature.unwrap_or(0.3),
        }))
        .send()
        .await
        .map_err(TranslateError::HttpError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TranslateError::ApiError { provider: "Azure".to_string(), status: status.as_u16(), message: body });
    }

    let json: serde_json::Value = response.json().await.map_err(|e| TranslateError::ApiError {
        provider: "Azure".to_string(), status: 0, message: e.to_string()
    })?;

    let translated_text = json["choices"][0]["message"]["content"].as_str().unwrap_or("").trim().to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0);
    let total_tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0);

    Ok(TranslationResult {
        provider_id: "azure".to_string(),
        provider_name: "Azure OpenAI".to_string(),
        source_text: request.text.clone(),
        translated_text,
        response_time_ms: start.elapsed().as_millis() as u64,
        is_success: true,
        error_message: None,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}

async fn translate_custom(
    client: &reqwest::Client,
    request: &TranslateRequest,
    saved: &Option<ProviderConfig>,
) -> Result<TranslationResult, TranslateError> {
    let cfg = resolve_config("custom", saved);
    if cfg.api_key.is_empty() {
        return Err(TranslateError::ApiKeyMissing("Custom".to_string()));
    }

    let url = format!("{}/chat/completions", build_base_url(&cfg.base_url));
    let start = std::time::Instant::now();

    let default_prompt = format!(
        "You are a translation engine. Your ONLY job is to translate text. Do NOT chat, explain, ask questions, or add commentary. Translate the user's input from {} to {}. Output ONLY the translated text, with no extra words, quotes, or formatting.",
        request.source_lang, request.target_lang
    );
    let system_prompt = request.system_prompt.as_deref().unwrap_or(&default_prompt);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": cfg.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": &request.text}
            ],
            "temperature": request.temperature.unwrap_or(0.3),
        }))
        .send()
        .await
        .map_err(TranslateError::HttpError)?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(TranslateError::ApiError { provider: "Custom".to_string(), status: status.as_u16(), message: body });
    }

    let json: serde_json::Value = response.json().await.map_err(|e| TranslateError::ApiError {
        provider: "Custom".to_string(), status: 0, message: e.to_string()
    })?;

    let translated_text = json["choices"][0]["message"]["content"].as_str().unwrap_or("").trim().to_string();

    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0);
    let total_tokens = json["usage"]["total_tokens"].as_u64().unwrap_or(0);

    Ok(TranslationResult {
        provider_id: "custom".to_string(),
        provider_name: "Custom".to_string(),
        source_text: request.text.clone(),
        translated_text,
        response_time_ms: start.elapsed().as_millis() as u64,
        is_success: true,
        error_message: None,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}