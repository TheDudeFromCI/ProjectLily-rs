use super::{MessageAction, SystemMessageSeverity};
use crate::llm::CompletionSettings;

#[derive(Debug, Clone)]
pub enum ChatMessage {
    System {
        severity: SystemMessageSeverity,
        content: String,
    },
    User {
        username: String,
        content: String,
    },
    Assistant {
        action: MessageAction,
        content: String,
    },
}

impl ChatMessage {
    pub fn get_role(&self) -> &'static str {
        match self {
            ChatMessage::System { .. } => "system",
            ChatMessage::User { .. } => "user",
            ChatMessage::Assistant { .. } => "assistant",
        }
    }

    pub fn get_content(&self) -> String {
        match self {
            ChatMessage::System { severity, content } => {
                format!("[{}] {}", severity, content)
            }

            ChatMessage::User { username, content } => {
                format!("{}: {}", username, content)
            }

            ChatMessage::Assistant { action, content } => {
                format!("{}: {}", action, content)
            }
        }
    }

    pub fn format(&self, settings: &CompletionSettings) -> String {
        match self {
            ChatMessage::System { .. } => {
                format!(
                    "{}{}{}",
                    settings.system_message_prefix,
                    self.get_content(),
                    settings.system_message_suffix
                )
            }

            ChatMessage::User { .. } => {
                format!(
                    "{}{}{}",
                    settings.user_message_prefix,
                    self.get_content(),
                    settings.user_message_suffix
                )
            }

            ChatMessage::Assistant { .. } => {
                format!(
                    "{}{}{}",
                    settings.assistant_message_prefix,
                    self.get_content(),
                    settings.assistant_message_suffix
                )
            }
        }
    }
}
