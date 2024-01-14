use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::AgentError;
use crate::llm::CompletionSettings;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSettings {
    pub name: String,
    pub creator: String,
    pub persona: String,
    pub directive: String,
    pub llm_options: CompletionSettings,
}

impl AgentSettings {
    pub fn from_file(file: &PathBuf) -> Result<Self, AgentError> {
        let contents = std::fs::read_to_string(file)?;
        let settings = serde_json::from_str(&contents)?;
        Ok(settings)
    }
}
