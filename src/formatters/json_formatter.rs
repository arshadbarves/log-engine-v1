use super::Formatter;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;

/// Formats log messages as JSON.
pub struct JsonFormatter;

#[async_trait]
impl Formatter for JsonFormatter {
    async fn format(&self, level: &str, message: &str, metadata: &serde_json::Value) -> String {
        let log = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "level": level,
            "message": message,
            "metadata": metadata,
        });
        log.to_string()
    }
}
