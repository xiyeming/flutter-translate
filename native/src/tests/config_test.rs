#[cfg(test)]
mod tests {
    use crate::ffi::types::*;
    use crate::config::ConfigManager;

    #[test]
    fn test_desktop_env_detection() {
        let env = ConfigManager::detect_desktop_env();
        assert!(matches!(
            env,
            DesktopEnv::KdePlasma
                | DesktopEnv::Hyprland
                | DesktopEnv::Gnome
                | DesktopEnv::Unknown
        ));
    }

    #[test]
    fn test_translate_error_display() {
        let err = crate::ffi::error::TranslateError::ProviderNotFound("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = crate::ffi::error::TranslateError::ApiKeyMissing("OpenAI".to_string());
        assert!(err.to_string().contains("OpenAI"));
    }

    #[test]
    fn test_config_error_display() {
        let err = crate::ffi::error::ConfigError::NotFound("test config".to_string());
        assert!(err.to_string().contains("test config"));

        let err = crate::ffi::error::ConfigError::KeyNotFound("api_key".to_string());
        assert!(err.to_string().contains("api_key"));

        let err = crate::ffi::error::ConfigError::ValidationError {
            field: "name".to_string(),
            message: "required".to_string(),
        };
        assert!(err.to_string().contains("name"));
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn test_provider_config_serialization() {
        let config = ProviderConfig {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            api_key: Some("secret".to_string()),
            api_url: None,
            model: "gpt-4o".to_string(),
            auth_type: "api_key".to_string(),
            is_active: true,
            sort_order: 0,
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test");
        assert_eq!(parsed.name, "Test Provider");
        assert_eq!(parsed.api_key, Some("secret".to_string()));
        assert_eq!(parsed.model, "gpt-4o");
    }

    #[test]
    fn test_translation_result_serialization() {
        let result = TranslationResult {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            source_text: "hello".to_string(),
            translated_text: "你好".to_string(),
            response_time_ms: 350,
            is_success: true,
            error_message: None,
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: TranslationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.translated_text, "你好");
        assert!(parsed.is_success);
    }

    #[test]
    fn test_shortcut_binding_serialization() {
        let binding = ShortcutBinding {
            id: "translate".to_string(),
            action: "translate_selected".to_string(),
            key_combination: "Super+Alt+F".to_string(),
            enabled: true,
        };

        let json = serde_json::to_string(&binding).unwrap();
        let parsed: ShortcutBinding = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key_combination, "Ctrl+Shift+T");
        assert!(parsed.enabled);
    }

    #[test]
    fn test_active_session_serialization() {
        let session = ActiveSession {
            last_provider_id: "openai".to_string(),
            last_compare_providers: vec!["openai".to_string(), "deepseek".to_string()],
            last_used: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&session).unwrap();
        let parsed: ActiveSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.last_provider_id, "openai");
        assert_eq!(parsed.last_compare_providers.len(), 2);
    }
}
