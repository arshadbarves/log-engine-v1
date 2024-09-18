pub mod json_formatter;
pub mod text_formatter;

use async_trait::async_trait;
use serde_json::Value;

/// Trait defining the interface for log message formatters.
#[async_trait]
pub trait Formatter: Send + Sync {
    /// Formats a log message based on level, message, and metadata.
    async fn format(&self, level: &str, message: &str, metadata: &Value) -> String;
}

pub use json_formatter::JsonFormatter;
pub use text_formatter::TextFormatter;
