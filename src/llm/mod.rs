use std::sync::Arc;

use thiserror::Error;

mod settings;

pub use settings::*;

pub mod llama_cpp;

#[derive(Clone)]
pub struct LlmWrapper {
    llm: Arc<Box<dyn LLM>>,
}

impl LlmWrapper {
    pub fn new(llm: Box<dyn LLM>) -> Self {
        Self { llm: Arc::new(llm) }
    }

    pub async fn validate_connection(&self) -> Result<(), LLMError> {
        self.llm.validate_connection().await
    }

    pub async fn query_completion(
        &self,
        prompt: String,
        settings: &CompletionSettings,
    ) -> Result<ChatResponse, LLMError> {
        self.llm.query_completion(prompt, settings).await
    }

    pub async fn tokenize(&self, text: String) -> Result<Vec<i32>, LLMError> {
        self.llm.tokenize(text).await
    }
}

#[async_trait::async_trait]
pub trait LLM: Send + Sync {
    async fn validate_connection(&self) -> Result<(), LLMError>;

    async fn query_completion(
        &self,
        prompt: String,
        settings: &CompletionSettings,
    ) -> Result<ChatResponse, LLMError>;

    async fn tokenize(&self, text: String) -> Result<Vec<i32>, LLMError>;
}

#[derive(Debug)]
pub struct ChatResponse {
    pub text: String,
    pub prompt_token_count: usize,
    pub generated_token_count: usize,
    pub generation_time: f64,
}

impl ChatResponse {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum LLMError {
    #[error("Failed to access server")]
    FailedToAccessServer,
    #[error("Model list is empty")]
    EmptyModelList,
    #[error("Failed to parse JSON response:\n===\n{json}\n===")]
    JsonParseError { json: String },
    #[error("Failed to serialize query")]
    FailedToSerializeQuery,
    #[error("Failed load the language model")]
    ModelNotLoaded,
    #[error("Server health returned within an unexpected state: {0}")]
    UnexpectedServerState(String),
    #[error("Failed to load grammar file at: {0}")]
    FailedToLoadGrammar(#[from] std::io::Error),
}
