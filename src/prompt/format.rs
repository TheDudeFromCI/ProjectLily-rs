use chrono::Local;
use itertools::Itertools;

use super::{ChatMessage, MessageAction, Subprocess, COMMAND_FORMAT, SYSTEM_PROMPT};
use crate::agent::Agent;
use crate::commands;
use crate::llm::CompletionSettings;

#[derive(Debug)]
pub struct PromptFormat {
    pub completion_settings: CompletionSettings,
    pub system_msg_prefix: String,
    pub system_msg_suffix: String,
    pub user_msg_prefix: String,
    pub user_msg_suffix: String,
    pub assistant_msg_prefix: String,
    pub assistant_msg_suffix: String,
    pub prompt_suffix_message: ChatMessage,
}

impl Default for PromptFormat {
    fn default() -> Self {
        Self {
            completion_settings: CompletionSettings::default(),
            system_msg_prefix: "### system\n".to_string(),
            system_msg_suffix: "\n".to_string(),
            user_msg_prefix: "### user\n".to_string(),
            user_msg_suffix: "\n".to_string(),
            assistant_msg_prefix: "### assistant\n".to_string(),
            assistant_msg_suffix: "\n".to_string(),
            prompt_suffix_message: ChatMessage::Assistant {
                process: Subprocess::InnerMonologue,
                action: MessageAction::Command,
                content: " ".to_string(),
            },
        }
    }
}

impl PromptFormat {
    pub fn update_subprocess(&mut self, process: Subprocess) {
        self.prompt_suffix_message = ChatMessage::Assistant {
            process,
            action: MessageAction::Command,
            content: " ".to_string(),
        };
    }

    pub fn get_system_prompt(&self, agent: &Agent) -> String {
        let time = &Local::now().format("%Y-%m-%d").to_string();
        let context_length = &self.completion_settings.context_length.to_string();
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

        SYSTEM_PROMPT
            .trim()
            .replace("{time}", time)
            .replace("{context_length}", context_length)
            .replace("{ai_name}", &agent.settings.name)
            .replace("{creator}", &agent.settings.creator)
            .replace("{command_list}", &command_list)
            .replace("{personality}", &agent.settings.persona)
            .replace("{memory_context}", memory_context)
            .replace("{primary_directive}", &agent.settings.directive)
    }

    pub fn format_chat_logs(&self, logs: &[&ChatMessage]) -> String {
        let prompt = logs
            .iter()
            .map(|l| match &l {
                ChatMessage::System { .. } => {
                    format!(
                        "{}{}{}",
                        self.system_msg_prefix,
                        l.get_content(),
                        self.system_msg_suffix
                    )
                }

                ChatMessage::User { .. } => {
                    format!(
                        "{}{}{}",
                        self.user_msg_prefix,
                        l.get_content(),
                        self.user_msg_suffix
                    )
                }

                ChatMessage::Assistant { .. } => format!(
                    "{}{}{}",
                    self.assistant_msg_prefix,
                    l.get_content(),
                    self.assistant_msg_suffix
                ),
            })
            .join("\n");

        let suffix = match self.prompt_suffix_message {
            ChatMessage::System { .. } => {
                format!(
                    "{}{}",
                    self.system_msg_prefix,
                    self.prompt_suffix_message.get_content(),
                )
            }

            ChatMessage::User { .. } => {
                format!(
                    "{}{}",
                    self.user_msg_prefix,
                    self.prompt_suffix_message.get_content(),
                )
            }

            ChatMessage::Assistant { .. } => format!(
                "{}{}",
                self.assistant_msg_prefix,
                self.prompt_suffix_message.get_content(),
            ),
        };

        format!("{}\n{}", prompt, suffix)
    }
}
