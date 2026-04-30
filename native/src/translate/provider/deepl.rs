use crate::ffi::error::TranslateError;
use crate::ffi::types::{TranslateRequest, TranslationResult};
use crate::translate::TranslateProvider;
use reqwest::Client;
use std::time::Instant;

pub struct DeepLProvider {
    client: Client,
    name: String,
    provider_id: String,
}

impl DeepLProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            name: "DeepL".to_string(),
            provider_id: "deepl".to_string(),
        }
    }

    fn get_api_key(&self) -> Option<String> {
        std::env::var("DEEPL_API_KEY").ok()
    }

    fn get_api_url(&self) -> String {
        match std::env::var("DEEPL_API_URL") {
            Ok(url) => url,
            Err(_) => "https://api-free.deepl.com/v2/translate".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl TranslateProvider for DeepLProvider {
    async fn translate(
        &self,
        request: &TranslateRequest,
    ) -> Result<TranslationResult, TranslateError> {
        let api_key = self
            .get_api_key()
            .ok_or_else(|| TranslateError::ApiKeyMissing("DeepL".to_string()))?;

        let api_url = self.get_api_url();
        let start = Instant::now();

        let source_lang = if request.source_lang == "auto" {
            None
        } else {
            Some(request.source_lang.to_uppercase())
        };

        let target_lang = request.target_lang.to_uppercase();

        let mut params = vec![
            ("text", request.text.clone()),
            ("target_lang", target_lang),
        ];

        if let Some(ref sl) = source_lang {
            params.push(("source_lang", sl.clone()));
        }

        let response = self
            .client
            .post(&api_url)
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| TranslateError::HttpError(e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranslateError::ApiError {
                provider: "DeepL".to_string(),
                status: status.as_u16(),
                message: body,
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TranslateError::ApiError {
                provider: "DeepL".to_string(),
                status: 0,
                message: e.to_string(),
            })?;

        let translated_text = json["translations"][0]["text"]
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
        let api_key = match self.get_api_key() {
            Some(key) => key,
            None => return Ok(false),
        };

        let response = self
            .client
            .post(&self.get_api_url())
            .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
            .form(&[("text", "test"), ("target_lang", "EN")])
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
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
        vec!["default".to_string(), "pro".to_string()]
    }
}
