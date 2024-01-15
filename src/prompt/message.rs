use std::fmt;

use itertools::Itertools;
use log::info;

use super::Subprocess;
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

    pub fn update_pre_prompt(&mut self, pre_prompt: String) {
        self.messages[0] = ChatMessage::System {
            severity: SystemMessageSeverity::Info,
            content: pre_prompt,
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

#[derive(Debug, Clone)]
pub enum ChatMessage {
    System {
        severity: SystemMessageSeverity,
        content: String,
    },
    User {
        username: String,
        action: MessageAction,
        content: String,
    },
    Assistant {
        process: Subprocess,
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

            ChatMessage::User {
                username,
                action,
                content,
            } => {
                format!("[{}] {}: {}", username, action, content)
            }

            ChatMessage::Assistant {
                process,
                action,
                content,
            } => {
                format!("[{}] {}: {}", process, action, content)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemMessageSeverity {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for SystemMessageSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemMessageSeverity::Debug => write!(f, "DEBUG"),
            SystemMessageSeverity::Info => write!(f, "INFO"),
            SystemMessageSeverity::Warn => write!(f, "WARN"),
            SystemMessageSeverity::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageAction {
    Command,
    Say,
    Think,
}

impl fmt::Display for MessageAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageAction::Command => write!(f, "COMMAND"),
            MessageAction::Say => write!(f, "SAY"),
            MessageAction::Think => write!(f, "THINK"),
        }
    }
}
