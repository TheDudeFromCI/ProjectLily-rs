use chrono::Local;
use log::{debug, info};

use super::{AgentError, AgentSettings, ProcessStateMachine};
use crate::communications::CommunicationManager;
use crate::llm::LlmWrapper;
use crate::mem_db::MemoryDB;
use crate::prompt::{ChatMessage, SYSTEM_PROMPT};

pub struct Agent {
    pub settings: AgentSettings,
    pub llm: LlmWrapper,
    pub mem_db: MemoryDB,
    pub communication_manager: CommunicationManager,
    pub process_state_machine: ProcessStateMachine,
}

impl Agent {
    pub async fn new(settings: AgentSettings, llm: LlmWrapper) -> Result<Self, AgentError> {
        let mut agent = Self {
            settings,
            llm,
            mem_db: MemoryDB::new().await?,
            communication_manager: CommunicationManager::default(),
            process_state_machine: ProcessStateMachine::default(),
        };
        agent.update_system_prompt().await?;

        Ok(agent)
    }

    pub async fn update(&mut self) -> Result<(), AgentError> {
        for mut message in self.communication_manager.receive_messages().await {
            self.update_token_count(&mut message).await?;
            self.mem_db.add_log_memory(message);
        }

        let response = self.query_llm().await?;
        self.log_message(response).await?;

        Ok(())
    }

    pub async fn update_token_count(&self, message: &mut ChatMessage) -> Result<(), AgentError> {
        if message.get_tokens().is_some() {
            return Ok(());
        }

        let content = message.format(&self.settings.llm_options);
        let tokens = self.llm.tokenize(content).await?;
        message.set_tokens(tokens.len());
        Ok(())
    }

    async fn query_llm(&mut self) -> Result<ChatMessage, AgentError> {
        let mut prompt = self.mem_db.get_log_prompt(&self.settings.llm_options);
        let action = self.process_state_machine.next_action();
        let prefix = action.as_prompt();

        prompt += &self.settings.llm_options.assistant_message_prefix;
        prompt += &prefix;
        self.settings.llm_options.grammar = Some(action.as_grammar());

        debug!(
            "Querying LLM with prompt:\n==========\n{}\n==========",
            &prompt
        );
        info!("Querying LLM with prompt prefix: `{}`", &prefix);

        let response = loop {
            let response = self
                .llm
                .query_completion(prompt.clone(), &self.settings.llm_options)
                .await?;

            if !response.is_empty() {
                break response.text;
            }

            debug!("LLM response was empty, retrying...");
        };

        info!("LLM response: {:?}", &response);

        let mut message = ChatMessage::Assistant {
            action: action.clone(),
            content: response,
            tokens: None,
        };
        self.update_token_count(&mut message).await?;

        Ok(message)
    }

    pub async fn update_system_prompt(&mut self) -> Result<(), AgentError> {
        let time = &Local::now().format("%Y-%m-%d").to_string();
        let memory_context = "None";

        let prompt = SYSTEM_PROMPT
            .trim()
            .replace("{time}", time)
            .replace("{ai_name}", &self.settings.name)
            .replace("{creator}", &self.settings.creator)
            .replace("{command_list}", "")
            .replace("{personality}", &self.settings.persona)
            .replace("{memory_context}", memory_context)
            .replace("{primary_directive}", &self.settings.directive);

        info!(
            "Updating system prompt.\n==========\n{}\n==========",
            &prompt
        );

        let tokens = self.llm.tokenize(prompt.clone()).await?.len();
        self.mem_db.update_pre_prompt(prompt, tokens);

        Ok(())
    }

    pub async fn log_message(&mut self, mut message: ChatMessage) -> Result<(), AgentError> {
        self.update_token_count(&mut message).await?;
        self.communication_manager.send_message(&message).await;
        self.mem_db.add_log_memory(message);

        Ok(())
    }
}
