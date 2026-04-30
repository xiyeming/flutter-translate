use crate::ffi::error::TranslateError;
use crate::ffi::types::{TranslateRequest, TranslationResult};
use crate::translate::TranslateProvider;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

pub struct CustomProvider {
    client: Client,
    name: String,
    provider_id: String,
}

impl CustomProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            name: "Custom".to_string(),
            provider_id: "custom".to_string(),
        }
    }

    fn get_api_key(&self) -> Option<String> {
        std::env::var("CUSTOM_API_KEY").ok()
    }

    fn get_base_url(&self) -> String {
        std::env::var("CUSTOM_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
            .trim_end_matches('/')
            .to_string()
    }

    fn get_chat_path(&self) -> String {
        std::env::var("CUSTOM_API_CHAT_PATH")
            .unwrap_or_else(|_| "/chat/completions".to_string())
    }

    fn get_api_url(&self) -> String {
        format!("{}{}", self.get_base_url(), self.get_chat_path())
    }

    fn get_auth_header(&self) -> String {
        std::env::var("CUSTOM_AUTH_HEADER")
            .unwrap_or_else(|_| "Authorization".to_string())
    }

    fn get_auth_prefix(&self) -> String {
        std::env::var("CUSTOM_AUTH_PREFIX")
            .unwrap_or_else(|_| "Bearer".to_string())
    }

    fn get_default_model(&self) -> String {
        std::env::var("CUSTOM_DEFAULT_MODEL")
            .unwrap_or_else(|_| "gpt-4o-mini".to_string())
    }
}

#[async_trait::async_trait]
impl TranslateProvider for CustomProvider {
    async fn translate(
        &self,
        request: &TranslateRequest,
    ) -> Result<TranslationResult, TranslateError> {
        let api_key = self
            .get_api_key()
            .ok_or_else(|| TranslateError::ApiKeyMissing("Custom".to_string()))?;

        let api_url = self.get_api_url();
        let start = Instant::now();

        let default_prompt = format!(
            "Translate from {} to {}. Only return the translated text.",
            request.source_lang, request.target_lang
        );
        let system_prompt = request.system_prompt.as_deref().unwrap_or(&default_prompt);
        let model = request.model.clone().unwrap_or_else(|| self.get_default_model());

        let auth_header = self.get_auth_header();
        let auth_value = format!("{} {}", self.get_auth_prefix(), api_key);

        let response = self
            .client
            .post(&api_url)
            .header(&auth_header, &auth_value)
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": &request.text}
                ],
                "temperature": request.temperature.unwrap_or(0.3),
            }))
            .send()
            .await
            .map_err(|e| TranslateError::HttpError(e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranslateError::ApiError {
                provider: "Custom".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TranslateError::ApiError {
                provider: "Custom".to_string(),
                status: 0,
                message: e.to_string(),
            })?;

        let translated_text = json["choices"][0]["message"]["content"]
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
            text: "test".to_string(),
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
            self.get_default_model(),
            "any-openai-compatible".to_string(),
        ]
    }
}
