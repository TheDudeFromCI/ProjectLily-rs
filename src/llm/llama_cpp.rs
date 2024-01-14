use std::time::Instant;

use itertools::Itertools;
use log::info;

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
        let url = format!("{}/health", self.url);

        loop {
            let response = reqwest::get(&url)
                .await
                .map_err(|_| LLMError::FailedToAccessServer)?;

            let res_text = response
                .text()
                .await
                .map_err(|_| LLMError::FailedToAccessServer)?;

            let res_json = json::parse(&res_text).map_err(|_| LLMError::JsonParseError {
                json: res_text.clone(),
            })?;

            let status = res_json["status"].as_str().unwrap_or("");

            match status {
                "ok" => break,
                "error" => return Err(LLMError::ModelNotLoaded),
                "loading_model" => {}
                state => return Err(LLMError::UnexpectedServerState(state.to_string())),
            }
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

        let grammar = match &settings.grammar {
            Some(grammar) => Some(std::fs::read_to_string(grammar)?),
            None => None,
        };

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
            grammar: grammar,
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
        let content = res_json["content"].as_str().unwrap_or("");

        info!("LLM Response: {} ({:.0} ms)", content, elapsed.as_millis());

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
