use std::fmt;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageAction {
    Query {
        question: Option<String>,
        answers: QueryAnswers,
    },
    SituationalAnalysis,
    EmotionalResponse,
    LogicalResponse,
    ProblemIdentification,
    GoalIdentification,
    ProblemSolving,
    EmotionalState,
    Command,
    Say,
}

impl MessageAction {
    pub const ALL: [MessageAction; 10] = [
        MessageAction::Query {
            question: None,
            answers: QueryAnswers::Boolean,
        },
        MessageAction::SituationalAnalysis,
        MessageAction::EmotionalResponse,
        MessageAction::LogicalResponse,
        MessageAction::ProblemIdentification,
        MessageAction::GoalIdentification,
        MessageAction::ProblemSolving,
        MessageAction::EmotionalState,
        MessageAction::Command,
        MessageAction::Say,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            MessageAction::Query { .. } => "QUERY",
            MessageAction::SituationalAnalysis => "SITUATIONAL_ANALYSIS",
            MessageAction::EmotionalResponse => "EMOTIONAL_RESPONSE",
            MessageAction::LogicalResponse => "LOGICAL_RESPONSE",
            MessageAction::ProblemIdentification => "PROBLEM_IDENTIFICATION",
            MessageAction::GoalIdentification => "GOAL_IDENTIFICATION",
            MessageAction::ProblemSolving => "PROBLEM_SOLVING",
            MessageAction::EmotionalState => "EMOTIONAL_STATE",
            MessageAction::Command => "COMMAND",
            MessageAction::Say => "SAY",
        }
    }

    pub fn as_prompt(&self) -> String {
        match self {
            MessageAction::Query { question, .. } => {
                format!("QUERY: {}\nANSWER: ", question.clone().unwrap_or_default())
            }
            s => {
                format!("{}: ", s.name())
            }
        }
    }

    pub fn as_grammar(&self) -> String {
        match self {
            MessageAction::Query { answers, .. } => answers.as_grammar(),
            _ => String::from(r#"root ::= [^ \t\n] [^\t\n]* "\n""#),
        }
    }

    pub fn get_explanation(&self) -> &str {
        match self {
            MessageAction::Query { .. } => {
                "When in this state, you will ask yourself a question, where your answer is used to determine your next action state."
            }
            MessageAction::SituationalAnalysis => {
                "When in this state, your goal is to analyze the current situation and observe as much information as possible about your current situation, especially the most recent events and messages."
            }
            MessageAction::ProblemIdentification => {
                "Analyze your logical and emotional responses to identify if there is currently a potential problem, and what that problem is. This state only identifies problems, it does not solve them."
            }
            MessageAction::EmotionalResponse => {
                "When in this state, try and respond emotionally to the current situation, if needed."
            }
            MessageAction::LogicalResponse => {
                "When in this state, try and respond logically to the current situation, if needed."
            }
            MessageAction::GoalIdentification => {
                "When in this state, identity the problem you are trying to solve, and define what your goal is. You do not need to determine how to solve the problem, just what the goal is. If there is no problem, then your goal may be assigned to any goal you wish to achieve."
            }
            MessageAction::ProblemSolving => {
                "When in this state, try and think of solutions to approach your current specified goal. Come up with as many solutions as is practical, and then determine which solution is the best."
            }
            MessageAction::EmotionalState => {
                "Analyze your emotional response, as well as ALL PAST emotional responses and emotional states to identify your current emotional state."
            }
            MessageAction::Command => {
                "When in this state, you may send a command, using natural language, to the interpreter, which will then be executed if possible."
            }
            MessageAction::Say => {
                "When in this state, you may say something, using natural language, to the user. This is the ONLY state where you may directly communicate with the user."
            }
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
