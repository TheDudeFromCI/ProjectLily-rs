use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageAction {
    Query {
        question: String,
        answers: Vec<String>,
    },
    SituationalAnalysis,
    ProblemIdentification,
    EmotionalResponse,
    LogicalResponse,
    EmotionalState,
    GoalIdentification,
    ProblemSolving,
    Command,
    Say,
}

impl MessageAction {
    pub fn name(&self) -> &'static str {
        match self {
            MessageAction::Query { .. } => "QUERY",
            MessageAction::SituationalAnalysis => "SITUATIONAL_ANALYSIS",
            MessageAction::ProblemIdentification => "PROBLEM_IDENTIFICATION",
            MessageAction::EmotionalResponse => "EMOTIONAL_RESPONSE",
            MessageAction::LogicalResponse => "LOGICAL_RESPONSE",
            MessageAction::EmotionalState => "EMOTIONAL_STATE",
            MessageAction::GoalIdentification => "GOAL_IDENTIFICATION",
            MessageAction::ProblemSolving => "PROBLEM_SOLVING",
            MessageAction::Command => "COMMAND",
            MessageAction::Say => "SAY",
        }
    }
}

impl fmt::Display for MessageAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
