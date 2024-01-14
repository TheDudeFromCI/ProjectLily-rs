use async_trait::async_trait;
use itertools::Itertools;

use super::{Command, CommandError, CommandExec};
use crate::agent::Agent;
use crate::prompt::{ChatMessage, SystemMessageSeverity};

pub struct RecallMemoryCommand;

#[async_trait]
impl Command for RecallMemoryCommand {
    fn name(&self) -> &str {
        "recall_memory"
    }

    fn args(&self) -> &[&str] {
        &["<search_terms>"]
    }

    fn description(&self) -> &str {
        "Recalls memories from your long-term memory database matching the search string."
    }

    fn usage(&self) -> &str {
        "recall_memory \"favorite color\""
    }

    async fn execute(&self, agent: &mut Agent, exec: CommandExec) -> Result<(), CommandError> {
        if exec.args.len() != 1 {
            return Err(CommandError::InvalidArguments(
                exec,
                "The `recall_memory` command expects exactly one argument".to_string(),
            ));
        }

        let memories = agent.mem_db.search(&exec.args[0], 3).map_err(|e| {
            CommandError::InternalError(exec.clone(), format!("Failed to save memory: {}", e))
        })?;

        let memories = format!(
            "Recalled the following memories:\n{}",
            memories
                .iter()
                .map(|m| format!("- `{}`", m.text))
                .join("\n")
        );

        agent.log_message(exec.clone().into()).await;
        agent
            .log_message(ChatMessage::System {
                severity: SystemMessageSeverity::Info,
                content: memories,
            })
            .await;

        Ok(())
    }
}
