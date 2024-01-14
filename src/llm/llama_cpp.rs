use std::time::Instant;

use itertools::Itertools;

use super::{ChatResponse, CompletionSettings, LLMError, LlmWrapper, LLM};

pub struct LlamaCppServer {
    pub url: String,
}

impl Default for LlamaCppServer {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LLM for LlamaCppServer {
    async fn validate_connection(&self) -> Result<(), LLMError> {
        let url = format!("{}/models", self.url);
        let response = reqwest::get(url)
            .await
            .map_err(|_| LLMError::FailedToAccessServer)?;

        let res_text = response
            .text()
            .await
            .map_err(|_| LLMError::FailedToAccessServer)?;

        let res_json = json::parse(&res_text).map_err(|_| LLMError::JsonParseError {
            json: res_text.clone(),
        })?;

        if res_json.is_empty() {
            return Err(LLMError::EmptyModelList);
        }

        Ok(())
    }

    async fn query_completion(
        &self,
        prompt: String,
        settings: &CompletionSettings,
    ) -> Result<ChatResponse, LLMError> {
        let logit_bias = format!(
            "[{}]",
            settings
                .logit_bias
                .iter()
                .map(|(k, v)| format!("[{},{}]", k, v))
                .join(",")
        );

        let json = json::object! {
            prompt: prompt,
            temperature: settings.temperature,
            top_k: settings.top_k,
            top_p: settings.top_p,
            min_p: settings.min_p,
            stop: settings.stop_tokens.clone(),
            repeat_penalty: settings.repeat_penalty,
            repeat_last_n: settings.repeat_last_n,
            presence_penalty: settings.presence_penalty,
            frequency_penalty: settings.frequency_penalty,
            logit_bias: logit_bias,
            cache_prompt: true,
            stream: false,
        };

        let url = format!("{}/completion", self.url);
        let time_start = Instant::now();

        let response = reqwest::Client::new()
            .post(url)
            .header("Content-Type", "application/json")
            .body(json.dump())
            .send()
            .await
            .map_err(|_| LLMError::FailedToAccessServer)?;

        let res_text = response
            .text()
            .await
            .map_err(|_| LLMError::FailedToAccessServer)?;

        let res_json = json::parse(&res_text).map_err(|_| LLMError::JsonParseError {
            json: res_text.clone(),
        })?;

        let elapsed = time_start.elapsed();

        Ok(ChatResponse {
            text: res_json["content"].to_string(),
            prompt_token_count: res_json["tokens_evaluated"].as_usize().unwrap_or(0),
            generated_token_count: res_json["tokens_predicted"].as_usize().unwrap_or(0),
            generation_time: elapsed.as_secs_f64(),
        })
    }
}

impl From<LlamaCppServer> for LlmWrapper {
    fn from(llm: LlamaCppServer) -> Self {
        Self::new(Box::new(llm))
    }
}
