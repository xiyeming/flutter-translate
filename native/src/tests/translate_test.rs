#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::ffi::error::TranslateError;
    use crate::ffi::types::{TranslateRequest, TranslationResult};
    use crate::translate::*;

    fn make_request() -> TranslateRequest {
        TranslateRequest {
            text: "Hello world".to_string(),
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            model: None,
            system_prompt: None,
            temperature: Some(0.0),
        }
    }

    #[test]
    fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(provider::openai::OpenAIProvider::new());
        registry.register(provider::deepl::DeepLProvider::new());
        registry.register(provider::google::GoogleProvider::new());

        assert_eq!(registry.count(), 3);
    }

    #[test]
    fn test_registry_get_provider() {
        let mut registry = ProviderRegistry::new();
        registry.register(provider::openai::OpenAIProvider::new());

        assert!(registry.get("openai").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_list() {
        let mut registry = ProviderRegistry::new();
        registry.register(provider::openai::OpenAIProvider::new());
        registry.register(provider::deepl::DeepLProvider::new());

        let providers = registry.list();
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn test_provider_names() {
        let openai = provider::openai::OpenAIProvider::new();
        assert_eq!(openai.name(), "OpenAI");
        assert_eq!(openai.provider_id(), "openai");
        assert!(!openai.supported_models().is_empty());

        let deepl = provider::deepl::DeepLProvider::new();
        assert_eq!(deepl.name(), "DeepL");
        assert_eq!(deepl.provider_id(), "deepl");

        let google = provider::google::GoogleProvider::new();
        assert_eq!(google.name(), "Google");
        assert_eq!(google.provider_id(), "google");
    }

    #[test]
    fn test_new_providers_names() {
        let qwen = provider::qwen::QwenProvider::new();
        assert_eq!(qwen.name(), "Qwen");
        assert!(qwen.supported_models().contains(&"qwen-turbo".to_string()));

        let deepseek = provider::deepseek::DeepSeekProvider::new();
        assert_eq!(deepseek.name(), "DeepSeek");
        assert!(deepseek.supported_models().contains(&"deepseek-chat".to_string()));

        let kimi = provider::kimi::KimiProvider::new();
        assert_eq!(kimi.name(), "Kimi");
        assert!(kimi.supported_models().contains(&"moonshot-v1-8k".to_string()));

        let glm = provider::glm::GLMProvider::new();
        assert_eq!(glm.name(), "GLM");
        assert!(glm.supported_models().contains(&"glm-4-plus".to_string()));
    }

    #[test]
    fn test_translate_error_display() {
        let err = TranslateError::ProviderNotFound("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = TranslateError::ApiKeyMissing("OpenAI".to_string());
        assert!(err.to_string().contains("OpenAI"));
    }

    #[test]
    fn test_router_provider_not_found() {
        let router = RequestRouter::new();
        // This test just ensures the router struct is properly created
        assert!(router.get_provider("openai").is_none());
    }

    #[tokio::test]
    async fn test_parallel_translator_empty_providers() {
        let router = Arc::new(RequestRouter::new());
        let translator = ParallelTranslator::new(router);
        let req = make_request();
        let result = translator.translate_compare(req, vec![]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_all_providers_registered() {
        let mut registry = ProviderRegistry::new();
        registry.register(provider::openai::OpenAIProvider::new());
        registry.register(provider::deepl::DeepLProvider::new());
        registry.register(provider::google::GoogleProvider::new());
        registry.register(provider::qwen::QwenProvider::new());
        registry.register(provider::deepseek::DeepSeekProvider::new());
        registry.register(provider::kimi::KimiProvider::new());
        registry.register(provider::glm::GLMProvider::new());

        assert_eq!(registry.count(), 7);

        let ids: Vec<&str> = registry.list().iter().map(|p| p.provider_id()).collect();
        assert!(ids.contains(&"openai"));
        assert!(ids.contains(&"deepl"));
        assert!(ids.contains(&"google"));
        assert!(ids.contains(&"qwen"));
        assert!(ids.contains(&"deepseek"));
        assert!(ids.contains(&"kimi"));
        assert!(ids.contains(&"glm"));
    }
}
