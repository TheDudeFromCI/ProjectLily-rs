use std::fmt;
use std::str::FromStr;

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

impl FromStr for SystemMessageSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(SystemMessageSeverity::Debug),
            "info" => Ok(SystemMessageSeverity::Info),
            "warn" => Ok(SystemMessageSeverity::Warn),
            "error" => Ok(SystemMessageSeverity::Error),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}
