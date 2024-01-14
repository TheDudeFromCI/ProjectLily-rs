use super::{AgentError, AgentSettings};
use crate::commands::{self};
use crate::communications::CommunicationManager;
use crate::llm::{ChatResponse, LlmWrapper};
use crate::mem_db::MemoryDB;
use crate::prompt::{ChatMessage, MessageLog, PromptFormat};

pub struct Agent {
    pub settings: AgentSettings,
    pub llm: LlmWrapper,
    pub mem_db: MemoryDB,
    pub log: MessageLog,
    pub prompt_format: PromptFormat,
    pub communication_manager: CommunicationManager,
}

impl Agent {
    pub async fn new(settings: AgentSettings, llm: LlmWrapper) -> Result<Self, AgentError> {
        Ok(Self {
            settings,
            llm,
            mem_db: MemoryDB::new().await?,
            log: MessageLog::default(),
            prompt_format: PromptFormat::default(),
            communication_manager: CommunicationManager::default(),
        })
    }

    pub async fn update(&mut self) -> Result<(), AgentError> {
        for message in self.communication_manager.receive_messages().await {
            self.log.add_message(message);
        }

        let response = loop {
            let response = self.query_llm().await?;
            if !response.is_empty() {
                break response;
            }
        };

        self.log.clear_temp();
        commands::execute(self, &response.text).await;

        Ok(())
    }

    async fn query_llm(&mut self) -> Result<ChatResponse, AgentError> {
        let pre_prompt = self.prompt_format.get_system_prompt(self);
        self.log.update_pre_prompt(pre_prompt);
        let messages = self.log.get_messages();

        let prompt = self.prompt_format.format_chat_logs(&messages);
        Ok(self
            .llm
            .query_completion(prompt, &self.prompt_format.completion_settings)
            .await?)
    }

    pub async fn log_message(&mut self, message: ChatMessage) {
        self.communication_manager.send_message(&message).await;
        self.log.add_message(message);
    }

    pub fn log_temp_message(&mut self, message: ChatMessage) {
        self.log.add_temp_message(message);
    }
}
