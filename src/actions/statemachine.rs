use super::MessageAction;

pub struct ProcessStateMachine {
    last_action: Option<MessageAction>,
}

impl ProcessStateMachine {
    pub fn new() -> Self {
        Self { last_action: None }
    }

    pub fn last_action(&self) -> Option<&MessageAction> {
        self.last_action.as_ref()
    }

    pub fn next_action(&mut self) -> &MessageAction {
        let next_action = match self.last_action {
            None => MessageAction::SituationalAnalysis,
            Some(MessageAction::SituationalAnalysis) => MessageAction::EmotionalResponse,
            Some(MessageAction::EmotionalResponse) => MessageAction::LogicalResponse,
            Some(MessageAction::LogicalResponse) => MessageAction::ProblemIdentification,
            Some(MessageAction::ProblemIdentification) => MessageAction::GoalIdentification,
            Some(MessageAction::GoalIdentification) => MessageAction::ProblemSolving,
            Some(MessageAction::ProblemSolving) => MessageAction::EmotionalState,
            Some(MessageAction::EmotionalState) => MessageAction::Command,
            Some(MessageAction::Command) => MessageAction::Say,
            Some(MessageAction::Say) => MessageAction::SituationalAnalysis,
            Some(MessageAction::Query { .. }) => MessageAction::SituationalAnalysis,
        };

        self.last_action = Some(next_action);
        self.last_action.as_ref().unwrap()
    }
}

impl Default for ProcessStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
