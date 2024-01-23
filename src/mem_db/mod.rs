mod log;
mod vector;

use rust_bert::RustBertError;
use thiserror::Error;
use tokio::task::JoinError;

use self::log::MessageLog;
use self::vector::VectorDB;
use crate::llm::CompletionSettings;
use crate::prompt::ChatMessage;

pub struct MemoryDB {
    log: MessageLog,
    vector: VectorDB,
}

impl MemoryDB {
    pub async fn new() -> Result<Self, MemoryDBError> {
        Ok(Self {
            log: MessageLog::new(),
            vector: VectorDB::new().await?,
        })
    }

    pub fn update_pre_prompt(&mut self, pre_prompt: String, tokens: usize) {
        self.log.update_pre_prompt(pre_prompt, tokens);
    }

    pub fn add_vector_memory(&mut self, message: &ChatMessage) -> Result<(), MemoryDBError> {
        let content = format!("{}:\n{}", message.get_role(), message.get_content());
        self.vector.add_memory(&content)
    }

    pub fn search_vector_memory(
        &self,
        query: &str,
        count: usize,
    ) -> Result<Vec<RecalledMemory>, MemoryDBError> {
        self.vector.search(query, count)
    }

    pub fn add_log_memory(&mut self, message: ChatMessage) {
        self.log.add_message(message);
    }

    pub fn get_log_prompt(&self, settings: &CompletionSettings) -> String {
        self.log.format(settings)
    }
}

#[derive(Debug)]
pub struct RecalledMemory {
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Error)]
pub enum MemoryDBError {
    #[error("Failed to create model: {0}")]
    FailedToCreateModel(#[from] RustBertError),
    #[error("Wrong embedding size: expected: {expected}, actual: {actual}")]
    WrongEmbeddingSize { expected: usize, actual: usize },
    #[error("Failed to search KD Tree: {0}")]
    KdTreeError(#[from] kdtree::ErrorKind),
    #[error("Failed to spawn blocking task: {0}")]
    AsyncError(#[from] JoinError),
}
