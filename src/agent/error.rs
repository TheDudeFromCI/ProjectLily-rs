use thiserror::Error;

use crate::llm::LLMError;
use crate::mem_db::MemoryDBError;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Failed to load agent settings from file: {0}")]
    AgentFileIO(#[from] std::io::Error),
    #[error("Failed to parse agent settings file: {0}")]
    AgentFileParse(#[from] serde_json::Error),
    #[error("An error has occurred within the Memory Database: {0}")]
    MemoryDBError(#[from] MemoryDBError),
    #[error("An error has occurred within the LLM: {0}")]
    LLMError(#[from] LLMError),
}
