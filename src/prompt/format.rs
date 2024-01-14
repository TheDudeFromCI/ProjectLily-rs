use chrono::Local;
use itertools::Itertools;

use super::{ChatMessage, MessageAction, Subprocess, COMMAND_FORMAT, SYSTEM_PROMPT};
use crate::agent::Agent;
use crate::commands;
use crate::llm::CompletionSettings;

#[derive(Debug)]
pub struct PromptFormat {
    pub prompt_suffix_message: ChatMessage,
}

impl Default for PromptFormat {
    fn default() -> Self {
        Self {
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
        let context_length = &agent.settings.llm_options.context_length.to_string();
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

    pub fn format_chat_logs(&self, logs: &[&ChatMessage], settings: &CompletionSettings) -> String {
        let prompt = logs
            .iter()
            .map(|l| match &l {
                ChatMessage::System { .. } => {
                    format!(
                        "{}{}{}",
                        settings.system_message_prefix,
                        l.get_content(),
                        settings.system_message_suffix
                    )
                }

                ChatMessage::User { .. } => {
                    format!(
                        "{}{}{}",
                        settings.user_message_prefix,
                        l.get_content(),
                        settings.user_message_suffix
                    )
                }

                ChatMessage::Assistant { .. } => format!(
                    "{}{}{}",
                    settings.assistant_message_prefix,
                    l.get_content(),
                    settings.assistant_message_suffix
                ),
            })
            .join("\n");

        let suffix = match self.prompt_suffix_message {
            ChatMessage::System { .. } => {
                format!(
                    "{}{}",
                    settings.system_message_prefix,
                    self.prompt_suffix_message.get_content(),
                )
            }

            ChatMessage::User { .. } => {
                format!(
                    "{}{}",
                    settings.user_message_prefix,
                    self.prompt_suffix_message.get_content(),
                )
            }

            ChatMessage::Assistant { .. } => format!(
                "{}{}",
                settings.assistant_message_prefix,
                self.prompt_suffix_message.get_content(),
            ),
        };

        format!("{}\n{}", prompt, suffix)
    }
}
