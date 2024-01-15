use std::time::Duration;

use async_trait::async_trait;

use super::{Command, CommandError, CommandExec};
use crate::agent::{Agent, Subprocess};
use crate::prompt::{ChatMessage, MessageAction};

pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    fn name(&self) -> &str {
        "say"
    }

    fn args(&self) -> &[&str] {
        &["<message>"]
    }

    fn description(&self) -> &str {
        "Say something out-loud to the user."
    }

    fn usage(&self) -> &str {
        "say \"Hello, world!\""
    }

    async fn execute(&self, agent: &mut Agent, exec: CommandExec) -> Result<(), CommandError> {
        if exec.args.len() != 1 {
            return Err(CommandError::InvalidArguments(
                exec,
                "The `say` command expects exactly one argument".to_string(),
            ));
        }

        agent.log_message(exec.clone().into()).await;
        agent
            .log_message(ChatMessage::Assistant {
                process: Subprocess::PublicSpeaker,
                action: MessageAction::Say,
                content: exec.args[0].clone(),
            })
            .await;

        tokio::time::sleep(Duration::from_secs_f32(10.0)).await;

        Ok(())
    }
}
