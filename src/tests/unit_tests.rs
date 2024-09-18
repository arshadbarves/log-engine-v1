#[cfg(test)]
mod unit_tests {
    use crate::config::ConfigurationManager;
    use crate::formatters::{Formatter, TextFormatter};
    use crate::handlers::{ConsoleHandler, LogHandler};
    use crate::metrics::MetricsManager;
    use crate::security::SecurityManager;
    use serde_json::json;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_configuration_loading() {
        let config = ConfigurationManager::new("config/config.yaml")
            .await
            .unwrap();
        let loaded_config = config.get_config().await;
        assert_eq!(loaded_config.level, "DEBUG");
        assert!(loaded_config.handlers.len() > 0);
    }

    #[tokio::test]
    async fn test_console_handler() {
        let handler = ConsoleHandler::new();
        let result = handler.emit("TEST [INFO] - Test message - {}").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_text_formatter() {
        let formatter = TextFormatter::new(Some("{level}: {message}".to_string()));
        let formatted = formatter
            .format("INFO", "Test message", &json!({"key": "value"}))
            .await;
        assert_eq!(formatted, "INFO: Test message");
    }

    #[tokio::test]
    async fn test_security_sanitization() {
        let security = SecurityManager::new(b"anexampleverysecurekey123456789012", None).unwrap();
        let sanitized = security.sanitize("User email is user@example.com");
        assert_eq!(sanitized, "User email is [REDACTED]");
    }

    #[tokio::test]
    async fn test_security_encryption_and_hashing() {
        let security = SecurityManager::new(b"anexampleverysecurekey123456789012", None).unwrap();
        let sanitized = "Test message".to_string();
        let encrypted = security.encrypt(&sanitized).unwrap();
        let hash = security.hash(&encrypted).unwrap();
        let integrity = security.verify_integrity(&encrypted, &hash).unwrap();
        assert!(integrity);
    }

    #[tokio::test]
    async fn test_metrics_initialization() {
        let metrics = MetricsManager::new();
        metrics.increment_log_count();
        metrics.increment_error();
        metrics.set_queue_size(5);
        assert_eq!(metrics.logs_processed.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.errors.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.queue_size.load(Ordering::SeqCst), 5);
    }
}
