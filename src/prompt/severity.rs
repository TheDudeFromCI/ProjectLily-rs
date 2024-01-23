use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemMessageSeverity {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for SystemMessageSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemMessageSeverity::Debug => write!(f, "DEBUG"),
            SystemMessageSeverity::Info => write!(f, "INFO"),
            SystemMessageSeverity::Warn => write!(f, "WARN"),
            SystemMessageSeverity::Error => write!(f, "ERROR"),
        }
    }
}
