use crate::ffi::error::TranslateError;
use crate::ffi::types::{TranslateRequest, TranslationResult};
use crate::translate::TranslateProvider;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

pub struct GoogleProvider {
    client: Client,
    name: String,
    provider_id: String,
}

impl GoogleProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            name: "Google".to_string(),
            provider_id: "google".to_string(),
        }
    }

    fn get_api_key(&self) -> Option<String> {
        std::env::var("GOOGLE_TRANSLATE_API_KEY").ok()
    }
}

#[async_trait::async_trait]
impl TranslateProvider for GoogleProvider {
    async fn translate(
        &self,
        request: &TranslateRequest,
    ) -> Result<TranslationResult, TranslateError> {
        let api_key = self
            .get_api_key()
            .ok_or_else(|| TranslateError::ApiKeyMissing("Google".to_string()))?;

        let api_url = format!(
            "https://translation.googleapis.com/language/translate/v2?key={}",
            api_key
        );

        let start = Instant::now();

        let response = self
            .client
            .post(&api_url)
            .json(&json!({
                "q": request.text,
                "source": request.source_lang,
                "target": request.target_lang,
                "format": "text",
            }))
            .send()
            .await
            .map_err(|e| TranslateError::HttpError(e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranslateError::ApiError {
                provider: "Google".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TranslateError::ApiError {
                provider: "Google".to_string(),
                status: 0,
                message: e.to_string(),
            })?;

        let translated_text = json["data"]["translations"][0]["translatedText"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let response_time_ms = start.elapsed().as_millis() as u64;

        Ok(TranslationResult {
            provider_id: self.provider_id.clone(),
            provider_name: self.name.clone(),
            source_text: request.text.clone(),
            translated_text,
            response_time_ms,
            is_success: true,
            error_message: None,
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        })
    }

    async fn test_connection(&self) -> Result<bool, TranslateError> {
        if self.get_api_key().is_none() {
            return Ok(false);
        }

        let test_request = TranslateRequest {
            text: "Hello".to_string(),
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            model: None,
            system_prompt: None,
            temperature: None,
        };

        match self.translate(&test_request).await {
            Ok(result) => Ok(result.is_success),
            Err(_) => Ok(false),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["nmt".to_string(), "base".to_string()]
    }
}
