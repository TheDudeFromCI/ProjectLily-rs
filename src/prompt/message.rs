use super::SystemMessageSeverity;
use crate::actions::MessageAction;
use crate::llm::CompletionSettings;

#[derive(Debug, Clone)]
pub enum ChatMessage {
    System {
        severity: SystemMessageSeverity,
        content: String,
        tokens: Option<usize>,
    },
    User {
        username: String,
        content: String,
        tokens: Option<usize>,
    },
    Assistant {
        action: MessageAction,
        content: String,
        tokens: Option<usize>,
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
            ChatMessage::System {
                severity, content, ..
            } => {
                format!("[{}] {}", severity, content)
            }

            ChatMessage::User {
                username, content, ..
            } => {
                format!("{}: {}", username, content)
            }

            ChatMessage::Assistant {
                action, content, ..
            } => {
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

    pub fn get_tokens(&self) -> Option<usize> {
        match self {
            ChatMessage::System { tokens, .. } => *tokens,
            ChatMessage::User { tokens, .. } => *tokens,
            ChatMessage::Assistant { tokens, .. } => *tokens,
        }
    }

    pub fn set_tokens(&mut self, tokens: usize) {
        match self {
            ChatMessage::System { tokens: t, .. } => *t = Some(tokens),
            ChatMessage::User { tokens: t, .. } => *t = Some(tokens),
            ChatMessage::Assistant { tokens: t, .. } => *t = Some(tokens),
        }
    }
}
