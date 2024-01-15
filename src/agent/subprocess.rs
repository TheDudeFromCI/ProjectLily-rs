use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subprocess {
    InnerMonologue,
    PublicSpeaker,
}

impl fmt::Display for Subprocess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subprocess::InnerMonologue => write!(f, "inner-monologue"),
            Subprocess::PublicSpeaker => write!(f, "public-speaker"),
        }
    }
}
