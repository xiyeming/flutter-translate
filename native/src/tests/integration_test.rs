#[cfg(test)]
mod tests {
    use crate::ffi::error::TranslateError;
    use crate::ffi::types::*;
    use crate::translate::*;

    #[test]
    fn test_router_provider_not_found_error() {
        let router = RequestRouter::new();
        let req = TranslateRequest {
            text: "test".to_string(),
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            model: None,
            system_prompt: None,
            temperature: None,
        };
        // Router without providers should fail
        assert!(router.get_provider("openai").is_none());
        let _ = req;
    }

    #[test]
    fn test_parallel_translator_structure() {
        let router = std::sync::Arc::new(RequestRouter::new());
        let translator = ParallelTranslator::new(std::sync::Arc::clone(&router));
        // Test that the translator is properly constructed
        drop(translator);
    }

    #[test]
    fn test_provider_registration_flow() {
        let mut registry = ProviderRegistry::new();

        // Register all 7 providers
        registry.register(provider::openai::OpenAIProvider::new());
        registry.register(provider::deepl::DeepLProvider::new());
        registry.register(provider::google::GoogleProvider::new());
        registry.register(provider::qwen::QwenProvider::new());
        registry.register(provider::deepseek::DeepSeekProvider::new());
        registry.register(provider::kimi::KimiProvider::new());
        registry.register(provider::glm::GLMProvider::new());

        assert_eq!(registry.count(), 7);

        // Verify each provider is accessible
        for id in &["openai", "deepl", "google", "qwen", "deepseek", "kimi", "glm"] {
            assert!(
                registry.get(id).is_some(),
                "Provider {} should be registered", id
            );
        }
    }

    #[test]
    fn test_provider_models_not_empty() {
        let providers: Vec<Box<dyn TranslateProvider>> = vec![
            Box::new(provider::openai::OpenAIProvider::new()),
            Box::new(provider::deepl::DeepLProvider::new()),
            Box::new(provider::google::GoogleProvider::new()),
            Box::new(provider::qwen::QwenProvider::new()),
            Box::new(provider::deepseek::DeepSeekProvider::new()),
            Box::new(provider::kimi::KimiProvider::new()),
            Box::new(provider::glm::GLMProvider::new()),
        ];

        for p in &providers {
            assert!(
                !p.supported_models().is_empty(),
                "{} should have at least one model", p.name()
            );
        }
    }

    #[test]
    fn test_translate_request_clone() {
        let req = TranslateRequest {
            text: "hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            model: Some("gpt-4o".to_string()),
            system_prompt: Some("Translate".to_string()),
            temperature: Some(0.5),
        };

        let cloned = req.clone();
        assert_eq!(cloned.text, req.text);
        assert_eq!(cloned.source_lang, req.source_lang);
        assert_eq!(cloned.target_lang, req.target_lang);
        assert_eq!(cloned.model, req.model);
        assert_eq!(cloned.system_prompt, req.system_prompt);
        assert_eq!(cloned.temperature, req.temperature);
    }

    #[test]
    fn test_translation_result_defaults() {
        let result = TranslationResult {
            provider_id: "test".to_string(),
            provider_name: "Test".to_string(),
            source_text: "src".to_string(),
            translated_text: "dst".to_string(),
            response_time_ms: 100,
            is_success: true,
            error_message: None,
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        assert!(result.is_success);
        assert_eq!(result.error_message, None);
    }

    #[test]
    fn test_translation_result_error() {
        let result = TranslationResult {
            provider_id: "test".to_string(),
            provider_name: "Test".to_string(),
            source_text: "src".to_string(),
            translated_text: String::new(),
            response_time_ms: 0,
            is_success: false,
            error_message: Some("API error".to_string()),
        };

        assert!(!result.is_success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_error_variants() {
        let errors = vec![
            TranslateError::NoAvailableProviders,
            TranslateError::RuntimeError,
            TranslateError::RateLimitExceeded,
            TranslateError::Timeout,
        ];

        for err in &errors {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_error_serde_conversion() {
        let err = TranslateError::ApiError {
            provider: "test".to_string(),
            status: 429,
            message: "rate limited".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("test"));
        assert!(s.contains("429"));
        assert!(s.contains("rate limited"));
    }

    #[test]
    fn test_config_error_variants() {
        let errors = vec![
            crate::ffi::error::ConfigError::NotFound("x".to_string()),
            crate::ffi::error::ConfigError::KeyNotFound("k".to_string()),
            crate::ffi::error::ConfigError::ValidationError {
                field: "f".to_string(),
                message: "m".to_string(),
            },
        ];

        for err in &errors {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_ocr_error_variants() {
        let errors = vec![
            crate::ffi::error::OcrError::UserCancelled,
            crate::ffi::error::OcrError::ScreenshotFailed,
            crate::ffi::error::OcrError::NoTextDetected,
            crate::ffi::error::OcrError::PermissionDenied,
        ];

        for err in &errors {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_clipboard_error_variants() {
        let errors = vec![
            crate::ffi::error::ClipboardError::Empty,
            crate::ffi::error::ClipboardError::ChannelError,
            crate::ffi::error::ClipboardError::PermissionDenied,
            crate::ffi::error::ClipboardError::WriteFailed,
        ];

        for err in &errors {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_tray_error_variants() {
        let errors = vec![
            crate::ffi::error::TrayError::InitFailed,
            crate::ffi::error::TrayError::IconNotFound,
            crate::ffi::error::TrayError::NotificationFailed,
        ];

        for err in &errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
