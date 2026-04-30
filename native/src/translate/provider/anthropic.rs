use crate::ffi::error::TranslateError;
use crate::ffi::types::{TranslateRequest, TranslationResult};
use crate::translate::TranslateProvider;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

pub struct AnthropicProvider {
    client: Client,
    name: String,
    provider_id: String,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            name: "Anthropic".to_string(),
            provider_id: "anthropic".to_string(),
        }
    }

    fn get_api_key(&self) -> Option<String> {
        std::env::var("ANTHROPIC_API_KEY").ok()
    }
}

#[async_trait::async_trait]
impl TranslateProvider for AnthropicProvider {
    async fn translate(
        &self,
        request: &TranslateRequest,
    ) -> Result<TranslationResult, TranslateError> {
        let api_key = self
            .get_api_key()
            .ok_or_else(|| TranslateError::ApiKeyMissing("Anthropic".to_string()))?;

        let api_url = "https://api.anthropic.com/v1/messages";
        let start = Instant::now();

        let sys_prompt = format!(
            "You are a professional translator. Translate the following user message from {} to {}. Return only the translated text.",
            request.source_lang, request.target_lang
        );
        let system_prompt = request.system_prompt.as_deref().unwrap_or(&sys_prompt);
        let model = request.model.as_deref().unwrap_or("claude-3-haiku-20240307");

        let response = self
            .client
            .post(api_url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": model,
                "system": system_prompt,
                "messages": [
                    {"role": "user", "content": &request.text}
                ],
                "temperature": request.temperature.unwrap_or(0.3),
                "max_tokens": 4096,
            }))
            .send()
            .await
            .map_err(|e| TranslateError::HttpError(e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranslateError::ApiError {
                provider: "Anthropic".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TranslateError::ApiError {
                provider: "Anthropic".to_string(),
                status: 0,
                message: e.to_string(),
            })?;

        let translated_text = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(TranslationResult {
            provider_id: self.provider_id.clone(),
            provider_name: self.name.clone(),
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

    async fn test_connection(&self) -> Result<bool, TranslateError> {
        if self.get_api_key().is_none() { return Ok(false); }
        let test = TranslateRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            model: None,
            system_prompt: None,
            temperature: Some(0.0),
        };
        Ok(self.translate(&test).await.map(|r| r.is_success).unwrap_or(false))
    }

    fn name(&self) -> &str { &self.name }
    fn provider_id(&self) -> &str { &self.provider_id }
    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-3-haiku-20240307".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-opus-20240229".to_string(),
        ]
    }
}
