use chrono::Local;
use itertools::Itertools;
use log::debug;

use super::{AgentError, AgentSettings};
use crate::commands::{self};
use crate::communications::CommunicationManager;
use crate::llm::{ChatResponse, LlmWrapper};
use crate::mem_db::MemoryDB;
use crate::prompt::{ChatMessage, MessageLog, COMMAND_FORMAT, SYSTEM_PROMPT};

pub struct Agent {
    pub settings: AgentSettings,
    pub llm: LlmWrapper,
    pub mem_db: MemoryDB,
    pub log: MessageLog,
    pub communication_manager: CommunicationManager,
}

impl Agent {
    pub async fn new(settings: AgentSettings, llm: LlmWrapper) -> Result<Self, AgentError> {
        let mut agent = Self {
            settings,
            llm,
            mem_db: MemoryDB::new().await?,
            log: MessageLog::default(),
            communication_manager: CommunicationManager::default(),
        };
        agent.update_system_prompt();

        Ok(agent)
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
        let prompt = self.log.format(&self.settings.llm_options);

        Ok(self
            .llm
            .query_completion(prompt, &self.settings.llm_options)
            .await?)
    }

    pub fn update_system_prompt(&mut self) {
        let time = &Local::now().format("%Y-%m-%d").to_string();
        let context_length = &self.settings.llm_options.context_length.to_string();
        let command_list = commands::COMMANDS
            .iter()
            .map(|c| {
                COMMAND_FORMAT
                    .trim()
                    .replace("{cmd_name}", c.name())
                    .replace("{args}", &c.args().join(" "))
                    .replace("{description}", c.description())
                    .replace("{example}", c.usage())
            })
            .join("\n");
        let memory_context = "None";

        let prompt = SYSTEM_PROMPT
            .trim()
            .replace("{time}", time)
            .replace("{context_length}", context_length)
            .replace("{ai_name}", &self.settings.name)
            .replace("{creator}", &self.settings.creator)
            .replace("{command_list}", &command_list)
            .replace("{personality}", &self.settings.persona)
            .replace("{memory_context}", memory_context)
            .replace("{primary_directive}", &self.settings.directive);

        debug!(
            "Updating system prompt.\n==========\n{}\n==========",
            &prompt
        );
        self.log.update_pre_prompt(prompt);
    }

    pub async fn log_message(&mut self, message: ChatMessage) {
        self.communication_manager.send_message(&message).await;
        self.log.add_message(message);
    }

    pub fn log_temp_message(&mut self, message: ChatMessage) {
        self.log.add_temp_message(message);
    }
}
