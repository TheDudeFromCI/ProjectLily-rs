use async_trait::async_trait;

use super::{Command, CommandError, CommandExec};
use crate::agent::{Agent, Subprocess};
use crate::prompt::{ChatMessage, MessageAction};

pub struct ThinkCommand;

#[async_trait]
impl Command for ThinkCommand {
    fn name(&self) -> &str {
        "think"
    }

    fn args(&self) -> &[&str] {
        &["<message>"]
    }

    fn description(&self) -> &str {
        "Think something to yourself."
    }

    fn usage(&self) -> &str {
        "think \"I wonder what I'm going to do, tomorrow.\""
    }

    async fn execute(&self, agent: &mut Agent, exec: CommandExec) -> Result<(), CommandError> {
        if exec.args.len() != 1 {
            return Err(CommandError::InvalidArguments(
                exec,
                "The `think` command expects exactly one argument".to_string(),
            ));
        }

        agent.log_message(exec.clone().into()).await;
        agent
            .log_message(ChatMessage::Assistant {
                process: Subprocess::InnerMonologue,
                action: MessageAction::Think,
                content: exec.args[0].clone(),
            })
            .await;

        Ok(())
    }
}
