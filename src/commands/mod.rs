use async_trait::async_trait;
use itertools::Itertools;
use thiserror::Error;

mod recall_memory;
mod save_memory;
mod say;
mod think;

pub use recall_memory::*;
pub use save_memory::*;
pub use say::*;
pub use think::*;

use crate::agent::Agent;
use crate::prompt::{ChatMessage, MessageAction, Subprocess, SystemMessageSeverity};

lazy_static! {
    pub static ref COMMANDS: [Box<dyn Command + Send + Sync>; 4] = [
        Box::new(SayCommand),
        Box::new(ThinkCommand),
        Box::new(SaveMemoryCommand),
        Box::new(RecallMemoryCommand),
    ];
}

#[derive(Debug, Clone)]
pub struct CommandExec {
    pub cmd_line: String,
    pub cmd: String,
    pub args: Vec<String>,
}

impl From<CommandExec> for ChatMessage {
    fn from(value: CommandExec) -> Self {
        ChatMessage::Assistant {
            process: Subprocess::InnerMonologue,
            action: MessageAction::Command,
            content: value.cmd_line,
        }
    }
}

pub fn get_command(name: &str) -> Option<&'static (dyn Command + Send + Sync)> {
    for command in COMMANDS.iter() {
        if command.name() == name {
            return Some(command.as_ref());
        }
    }

    None
}

pub async fn execute(agent: &mut Agent, cmd: &str) {
    match try_run_cmd(agent, cmd).await {
        Ok(_) => {}
        Err(err) => {
            agent.log_temp_message(ChatMessage::Assistant {
                process: Subprocess::InnerMonologue,
                action: MessageAction::Command,
                content: cmd.to_string(),
            });

            print_cmd_err(agent, err);
        }
    }
}

async fn try_run_cmd(agent: &mut Agent, cmd: &str) -> Result<(), CommandError> {
    let elements = shlex::split(cmd).ok_or_else(|| CommandError::CommandFormat(cmd.to_string()))?;

    let cmd = elements
        .first()
        .ok_or(CommandError::EmptyCommand)?
        .to_owned()
        .to_lowercase();

    let args = elements.into_iter().skip(1).collect();

    let exec = CommandExec {
        cmd_line: cmd.clone(),
        cmd: cmd.clone(),
        args,
    };

    for command in COMMANDS.iter() {
        if command.name() == cmd {
            return command.execute(agent, exec).await;
        }
    }

    Err(CommandError::InvalidCommand(cmd.to_string()))
}

fn print_cmd_err(agent: &mut Agent, err: CommandError) {
    match err {
        CommandError::InvalidCommand(cmd) => {
            let content = format!(
                "You have entered an invalid command: `{}`. Available commands: {}",
                cmd,
                COMMANDS
                    .iter()
                    .map(|c| format!("`{}`", c.name()))
                    .join(", ")
            );

            agent.log_temp_message(ChatMessage::System {
                severity: SystemMessageSeverity::Error,
                content,
            })
        }

        CommandError::InvalidArguments(exec, message) => {
            let content = format!(
                "You have used the command `{}` incorrectly. {}\nExample usage: `{}`",
                exec.cmd,
                message,
                get_command(&exec.cmd).unwrap().usage()
            );

            agent.log_temp_message(ChatMessage::System {
                severity: SystemMessageSeverity::Error,
                content,
            })
        }

        CommandError::InternalError(exec, message) => {
            let content = format!(
                "An internal error has occurred while executing the command `{}`. {}",
                exec.cmd, message
            );

            agent.log_temp_message(ChatMessage::System {
                severity: SystemMessageSeverity::Error,
                content,
            })
        }

        CommandError::CommandFormat(_) => {
            let content = "The command you entered could not be parsed. Make sure you are using quotes and whitespace correctly.".to_string();
            agent.log_temp_message(ChatMessage::System {
                severity: SystemMessageSeverity::Error,
                content,
            })
        }

        CommandError::EmptyCommand => {
            let content = "The command you entered was empty.".to_string();
            agent.log_temp_message(ChatMessage::System {
                severity: SystemMessageSeverity::Error,
                content,
            })
        }
    }
}

#[async_trait]
pub trait Command {
    fn name(&self) -> &str;
    fn args(&self) -> &[&str];
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, agent: &mut Agent, exec: CommandExec) -> Result<(), CommandError>;
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Command does not exist: {0}")]
    InvalidCommand(String),
    #[error("Invalid arguments for {0:?}.\nError: {1}")]
    InvalidArguments(CommandExec, String),
    #[error("Internal error: `{1}` for the command {1}.")]
    InternalError(CommandExec, String),
    #[error("Command string could not be parsed: {0}")]
    CommandFormat(String),
    #[error("Command string cannot be empty")]
    EmptyCommand,
}
