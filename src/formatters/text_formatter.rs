use super::Formatter;
use async_trait::async_trait;
use chrono::Utc;

/// Formats log messages as plain text.
pub struct TextFormatter {
    pattern: String,
}

impl TextFormatter {
    /// Initializes the TextFormatter with a specific pattern.
    pub fn new(pattern: Option<String>) -> Self {
        // Default pattern if none provided
        let default_pattern = "{timestamp} [{level}] - {message} - {metadata}".to_string();
        TextFormatter {
            pattern: pattern.unwrap_or(default_pattern),
        }
    }
}

#[async_trait]
impl Formatter for TextFormatter {
    async fn format(&self, level: &str, message: &str, metadata: &serde_json::Value) -> String {
        let timestamp = Utc::now().to_rfc3339();
        let metadata_str = metadata.to_string();
        self.pattern
            .replace("{timestamp}", &timestamp)
            .replace("{level}", level)
            .replace("{message}", message)
            .replace("{metadata}", &metadata_str)
    }
}
