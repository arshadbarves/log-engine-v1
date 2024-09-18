use super::LogHandler;
use async_trait::async_trait;

/// Handles console output for log messages.
pub struct ConsoleHandler {
    // Configuration fields if needed, e.g., color schemes
}

impl ConsoleHandler {
    /// Initializes the ConsoleHandler.
    pub fn new() -> Self {
        ConsoleHandler {}
    }
}

#[async_trait]
impl LogHandler for ConsoleHandler {
    async fn emit(&self, formatted: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Simple color-coding based on log level
        if let Some(start) = formatted.find('[') {
            if let Some(end) = formatted[start..].find(']') {
                let level = &formatted[start + 1..start + end];
                let colored_msg = match level {
                    "DEBUG" => format!("\x1b[32m{}\x1b[0m", formatted), // Green
                    "INFO" => format!("\x1b[34m{}\x1b[0m", formatted),  // Blue
                    "WARN" => format!("\x1b[33m{}\x1b[0m", formatted),  // Yellow
                    "ERROR" => format!("\x1b[31m{}\x1b[0m", formatted), // Red
                    "FATAL" => format!("\x1b[41;37m{}\x1b[0m", formatted), // White on Red
                    _ => formatted.to_string(),
                };
                println!("{}", colored_msg);
            } else {
                println!("{}", formatted);
            }
        } else {
            println!("{}", formatted);
        }
        Ok(())
    }
}
