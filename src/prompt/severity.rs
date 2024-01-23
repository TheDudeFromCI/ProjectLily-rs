use std::fmt;
use std::str::FromStr;

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

impl FromStr for SystemMessageSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" | "DEBUG" => Ok(SystemMessageSeverity::Debug),
            "info" | "INFO" => Ok(SystemMessageSeverity::Info),
            "warn" | "WARN" => Ok(SystemMessageSeverity::Warn),
            "error" | "ERROR" => Ok(SystemMessageSeverity::Error),
            _ => Err(format!("Invalid severity: {}", s)),
        }
    }
}
