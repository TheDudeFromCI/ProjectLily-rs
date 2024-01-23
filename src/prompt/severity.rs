use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemMessageSeverity {
    Debug,
    Info,
    Warn,
    Error,
}

impl SystemMessageSeverity {
    pub fn name(&self) -> &'static str {
        match self {
            SystemMessageSeverity::Debug => "debug",
            SystemMessageSeverity::Info => "info",
            SystemMessageSeverity::Warn => "warn",
            SystemMessageSeverity::Error => "error",
        }
    }
}

impl fmt::Display for SystemMessageSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
