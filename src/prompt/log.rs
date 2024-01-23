use itertools::Itertools;
use log::info;

use super::{ChatMessage, SystemMessageSeverity};
use crate::llm::CompletionSettings;

pub struct MessageLog {
    messages: Vec<ChatMessage>,
    temp_messages: Vec<ChatMessage>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage::System {
                severity: SystemMessageSeverity::Info,
                content: "Pre-Prompt Placeholder".to_string(),
                tokens: None,
            }],
            temp_messages: Vec::new(),
        }
    }

    pub fn clear_log(&mut self) {
        self.messages.truncate(1);
        self.temp_messages.clear();
    }

    pub fn clear_temp(&mut self) {
        self.temp_messages.clear();
    }

    pub fn update_pre_prompt(&mut self, pre_prompt: String, tokens: usize) {
        self.messages[0] = ChatMessage::System {
            severity: SystemMessageSeverity::Info,
            content: pre_prompt,
            tokens: Some(tokens),
        };
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        info!("{} : {}", message.get_role(), message.get_content());
        self.messages.push(message);
    }

    pub fn add_temp_message(&mut self, message: ChatMessage) {
        self.temp_messages.push(message);
    }

    pub fn get_messages(&self) -> Vec<&ChatMessage> {
        let count = self.messages.len() + self.temp_messages.len();
        let mut message_pointers = Vec::with_capacity(count);

        for message in &self.messages {
            message_pointers.push(message);
        }

        for message in &self.temp_messages {
            message_pointers.push(message);
        }

        message_pointers
    }

    pub fn format(&self, settings: &CompletionSettings) -> String {
        self.messages
            .iter()
            .chain(self.temp_messages.iter())
            .map(|l| l.format(settings))
            .join("")
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}
