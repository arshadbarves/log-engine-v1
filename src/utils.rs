use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum LogLevel {
    TRACE = 0,
    DEBUG,
    INFO,
    WARN,
    ERROR,
    FATAL,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl LogLevel {
    pub fn from_str(level: &str) -> Option<Self> {
        match level.to_uppercase().as_str() {
            "TRACE" => Some(LogLevel::TRACE),
            "DEBUG" => Some(LogLevel::DEBUG),
            "INFO" => Some(LogLevel::INFO),
            "WARN" => Some(LogLevel::WARN),
            "ERROR" => Some(LogLevel::ERROR),
            "FATAL" => Some(LogLevel::FATAL),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::TRACE => "TRACE",
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::FATAL => "FATAL",
        }
    }
}