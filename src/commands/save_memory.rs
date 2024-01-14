use async_trait::async_trait;

use super::{Command, CommandError, CommandExec};
use crate::agent::Agent;
use crate::prompt::{ChatMessage, SystemMessageSeverity};

pub struct SaveMemoryCommand;

#[async_trait]
impl Command for SaveMemoryCommand {
    fn name(&self) -> &str {
        "save_memory"
    }

    fn args(&self) -> &[&str] {
        &["<memory>"]
    }

    fn description(&self) -> &str {
        "Saves a memory to your long-term memory database."
    }

    fn usage(&self) -> &str {
        "save_memory \"This is something I want to remember forever.\""
    }

    async fn execute(&self, agent: &mut Agent, exec: CommandExec) -> Result<(), CommandError> {
        if exec.args.len() != 1 {
            return Err(CommandError::InvalidArguments(
                exec,
                "The `save_memory` command expects exactly one argument".to_string(),
            ));
        }

        agent.mem_db.add_memory(&exec.args[0]).map_err(|e| {
            CommandError::InternalError(exec.clone(), format!("Failed to save memory: {}", e))
        })?;

        agent.log_message(exec.clone().into()).await;
        agent
            .log_message(ChatMessage::System {
                severity: SystemMessageSeverity::Info,
                content: "This memory has been saved.".to_string(),
            })
            .await;

        Ok(())
    }
}
