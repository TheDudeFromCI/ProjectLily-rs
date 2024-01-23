use std::fmt;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageAction {
    Query {
        question: String,
        answers: QueryAnswers,
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

    pub fn as_prompt(&self) -> String {
        match self {
            MessageAction::Query { question, .. } => {
                format!("QUERY: {}\nANSWER: ", question)
            }
            s => {
                format!("{}: ", s.name())
            }
        }
    }

    pub fn as_grammar(&self) -> String {
        match self {
            MessageAction::Query { answers, .. } => answers.as_grammar(),
            _ => String::from(r#"root ::= [^\n]+ "\n""#),
        }
    }
}

impl fmt::Display for MessageAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryAnswers {
    Literals(Vec<String>),
    String,
    Boolean,
    Number,
}

impl QueryAnswers {
    pub fn as_grammar(&self) -> String {
        match self {
            QueryAnswers::Literals(answers) => {
                let answer = answers.iter().map(|s| format!("\"{}\"", s)).join(" | ");
                format!(r#"root ::= ({}) "\n""#, answer)
            }
            QueryAnswers::String => String::from(r#"root ::= [^\n]+ "\n""#),
            QueryAnswers::Boolean => String::from(r#"root ::= ("Yes" | "No") "\n""#),
            QueryAnswers::Number => String::from(r#"root ::= [0-9]+ "\n""#),
        }
    }
}
