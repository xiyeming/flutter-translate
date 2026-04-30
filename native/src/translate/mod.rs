pub mod engine;
pub mod router;
pub mod provider;

use crate::ffi::error::TranslateError;
use crate::ffi::types::{TranslateRequest, TranslationResult};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait TranslateProvider: Send + Sync {
    async fn translate(
        &self,
        request: &TranslateRequest,
    ) -> Result<TranslationResult, TranslateError>;

    async fn test_connection(&self) -> Result<bool, TranslateError>;

    fn name(&self) -> &str;

    fn provider_id(&self) -> &str;

    fn supported_models(&self) -> Vec<String>;
}

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn TranslateProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<P: TranslateProvider + 'static>(&mut self, provider: P) {
        let id = provider.provider_id().to_string();
        self.providers.insert(id, Box::new(provider));
    }

    pub fn get(&self, provider_id: &str) -> Option<&dyn TranslateProvider> {
        self.providers.get(provider_id).map(|p| p.as_ref())
    }

    pub fn list(&self) -> Vec<&dyn TranslateProvider> {
        self.providers.values().map(|p| p.as_ref()).collect()
    }

    pub fn count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RequestRouter {
    registry: ProviderRegistry,
}

impl RequestRouter {
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::new(),
        }
    }

    pub fn with_registry(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ProviderRegistry {
        &mut self.registry
    }

    pub async fn route(
        &self,
        request: &TranslateRequest,
        provider_id: &str,
    ) -> Result<TranslationResult, TranslateError> {
        let provider = self
            .registry
            .get(provider_id)
            .ok_or_else(|| TranslateError::ProviderNotFound(provider_id.to_string()))?;

        provider.translate(request).await
    }

    pub fn get_provider(&self, provider_id: &str) -> Option<&dyn TranslateProvider> {
        self.registry.get(provider_id)
    }
}

pub struct ParallelTranslator {
    router: Arc<RequestRouter>,
}

impl ParallelTranslator {
    pub fn new(router: Arc<RequestRouter>) -> Self {
        Self { router }
    }

    pub async fn translate_compare(
        &self,
        request: TranslateRequest,
        provider_ids: Vec<String>,
    ) -> Result<Vec<TranslationResult>, TranslateError> {
        if provider_ids.is_empty() {
            return Err(TranslateError::NoAvailableProviders);
        }

        let request = Arc::new(request);
        let mut handles = Vec::new();

        for provider_id in provider_ids {
            let router = Arc::clone(&self.router);
            let req = Arc::clone(&request);
            let pid = provider_id;

            handles.push(tokio::spawn(async move {
                router.route(&req, &pid).await
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
}

static ROUTER: once_cell::sync::Lazy<tokio::sync::RwLock<Arc<RequestRouter>>> =
    once_cell::sync::Lazy::new(|| {
        tokio::sync::RwLock::new(Arc::new(RequestRouter::new()))
    });

pub async fn init_router() {
    let mut guard = ROUTER.write().await;
    let mut registry = ProviderRegistry::new();

    registry.register(provider::openai::OpenAIProvider::new());
    registry.register(provider::deepl::DeepLProvider::new());
    registry.register(provider::google::GoogleProvider::new());
    registry.register(provider::qwen::QwenProvider::new());
    registry.register(provider::deepseek::DeepSeekProvider::new());
    registry.register(provider::kimi::KimiProvider::new());
    registry.register(provider::glm::GLMProvider::new());
    registry.register(provider::anthropic::AnthropicProvider::new());
    registry.register(provider::azure::AzureProvider::new());
    registry.register(provider::custom::CustomProvider::new());

    *guard = Arc::new(RequestRouter::with_registry(registry));
}

pub async fn get_router() -> Arc<RequestRouter> {
    let guard = ROUTER.read().await;
    Arc::clone(&guard)
}

pub async fn get_parallel_translator() -> ParallelTranslator {
    let router = get_router().await;
    ParallelTranslator::new(router)
}
