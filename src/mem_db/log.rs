use itertools::Itertools;
use log::info;

use crate::llm::CompletionSettings;
use crate::prompt::{ChatMessage, SystemMessageSeverity};

pub struct MessageLog {
    messages: Vec<ChatMessage>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage::System {
                severity: SystemMessageSeverity::Info,
                content: "Pre-Prompt Placeholder".to_string(),
                tokens: None,
            }],
        }
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

    pub fn format(&self, settings: &CompletionSettings) -> String {
        self.messages.iter().map(|l| l.format(settings)).join("")
    }
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}
