use std::fmt::Display;
use crate::config::ConfigurationManager;
use crate::formatters::Formatter;
use crate::handlers::LogHandler;
use crate::metrics::MetricsManager;
use crate::security::SecurityManager;
use crate::utils::LogLevel;
use chrono::Utc;
use crossbeam::queue::SegQueue;
use serde_json::Value;
use std::sync::Arc;
use std::{fmt, thread_local};
use thiserror::Error;
use tokio::sync::Notify;
use tokio::task;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("Handler error: {0}")]
    HandlerError(String),
    #[error("Formatter error: {0}")]
    FormatterError(String),
    #[error("Security error: {0}")]
    SecurityError(String),
}

/// Represents a log message with associated metadata.
pub struct LogMessage {
    pub id: Uuid,
    pub level: LogLevel,
    pub message: String,
    pub metadata: Value,
    pub timestamp: String,
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LogMessage {{ id: {}, level: {:?}, message: {}, metadata: {}, timestamp: {} }}",
            self.id, self.level, self.message, self.metadata, self.timestamp
        )
    }
}

/// Core Logger struct managing the logging process.
pub struct Logger {
    config_manager: Arc<ConfigurationManager>,
    handlers: Vec<Arc<dyn LogHandler>>,
    formatter: Arc<dyn Formatter>,
    queue: Arc<SegQueue<LogMessage>>,
    notify: Arc<Notify>,
    pub metrics: Arc<MetricsManager>,
    security: Arc<SecurityManager>,
}

impl Logger {
    /// Initializes the Logger with configuration and security key.
    pub async fn new(config_file: &str, security_key: &[u8]) -> Result<Arc<Self>, LoggerError> {
        let config_manager = Arc::new(
            ConfigurationManager::new(config_file)
                .await
                .map_err(|e| LoggerError::FormatterError(e.to_string()))?,
        );
        let config = config_manager.get_config().await;

        // Initialize handlers based on config
        let mut handlers: Vec<Arc<dyn LogHandler>> = Vec::new();
        for handler_cfg in config.handlers {
            match handler_cfg.type_.as_str() {
                "console" => handlers.push(Arc::new(crate::handlers::ConsoleHandler::new())),
                "file" => {
                    let file_path = handler_cfg
                        .config
                        .as_ref()
                        .and_then(|cfg| cfg.get("file_path"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("logs/app.log")
                        .to_string();
                    let max_size = handler_cfg
                        .config
                        .as_ref()
                        .and_then(|cfg| cfg.get("max_size"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10 * 1024 * 1024);
                    handlers.push(Arc::new(crate::handlers::FileHandler::new(
                        file_path.into(),
                        max_size,
                    )));
                }
                "remote" => {
                    let address = handler_cfg
                        .config
                        .as_ref()
                        .and_then(|cfg| cfg.get("address"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("127.0.0.1")
                        .to_string();
                    let port = handler_cfg
                        .config
                        .as_ref()
                        .and_then(|cfg| cfg.get("port"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(9000) as u16;
                    let retries = handler_cfg
                        .config
                        .as_ref()
                        .and_then(|cfg| cfg.get("retries"))
                        .and_then(|v| v.as_u64())
                        .map(|v| v as usize);
                    handlers.push(Arc::new(crate::handlers::RemoteHandler::new(
                        address, port, retries,
                    )));
                }
                "memory" => {
                    let capacity = handler_cfg
                        .config
                        .as_ref()
                        .and_then(|cfg| cfg.get("capacity"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1000) as usize;
                    handlers.push(Arc::new(crate::handlers::MemoryHandler::new(capacity)));
                }
                _ => continue,
            }
        }

        // Initialize formatter
        let formatter: Arc<dyn Formatter> = match config.formatter.as_deref() {
            Some("json") => Arc::new(crate::formatters::JsonFormatter),
            Some("text") => Arc::new(crate::formatters::TextFormatter::new(None)),
            _ => Arc::new(crate::formatters::TextFormatter::new(None)),
        };

        // Initialize security manager
        let security = Arc::new(
            SecurityManager::new(security_key, None)
                .map_err(|e| LoggerError::SecurityError(e.to_string()))?,
        );

        // Initialize metrics
        let metrics = Arc::new(MetricsManager::new());

        // Initialize lock-free queue
        let queue = Arc::new(SegQueue::new());

        // Initialize notify for worker
        let notify = Arc::new(Notify::new());

        let logger = Arc::new(Logger {
            config_manager: config_manager.clone(),
            handlers,
            formatter,
            queue: queue.clone(),
            notify: notify.clone(),
            metrics,
            security,
        });

        // Initialize thread-local buffer
        thread_local! {
            static BUFFER: task::LocalSet = task::LocalSet::new();
        }

        // Start the worker task
        Logger::start_worker(logger.clone());

        Ok(logger)
    }

    /// Starts the asynchronous logging worker that processes log messages from the queue.
    fn start_worker(logger: Arc<Logger>) {
        let queue = logger.queue.clone();
        let notify = logger.notify.clone();
        let handlers = logger.handlers.clone();
        let formatter = logger.formatter.clone();
        let metrics = logger.metrics.clone();
        let security = logger.security.clone();

        task::spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    // Wait for notification or check queue periodically
                    tokio::select! {
                        _ = notify.notified() => {},
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {},
                    }

                    let mut batch = Vec::new();
                    while let Some(log) = queue.pop() {
                        batch.push(log);
                    }

                    if !batch.is_empty() {
                        for log in batch {
                            // Security: sanitize, encrypt, and hash
                            let sanitized = security.sanitize(&log.message);
                            let encrypted = match security.encrypt(&sanitized) {
                                Ok(enc) => enc,
                                Err(e) => {
                                    metrics.increment_error();
                                    eprintln!("Encryption failed: {}", e);
                                    continue;
                                }
                            };
                            let hash = match security.hash(&encrypted) {
                                Ok(h) => h,
                                Err(e) => {
                                    metrics.increment_error();
                                    eprintln!("Hashing failed: {}", e);
                                    continue;
                                }
                            };

                            let metadata = serde_json::json!({
                                "hash": hash,
                                "timestamp": log.timestamp,
                                "metadata": log.metadata,
                            });

                            // Format the log
                            let formatted = formatter
                                .format(&log.level.to_string(), &encrypted, &metadata)
                                .await;

                            // Emit to all handlers
                            for handler in &handlers {
                                let emit_result = handler.emit(&formatted).await;
                                if emit_result.is_err() {
                                    metrics.increment_error();
                                    eprintln!("Handler emit failed: {:?}", emit_result.err());
                                }
                            }

                            // Update metrics
                            metrics.increment_log_count();
                            // Optionally, record latency or other metrics
                        }

                        // Update queue size metric
                        metrics.set_queue_size(queue.len());
                    }
                }
            });
        });
    }

    /// Enqueues a log message for processing.
    pub fn log(&self, level: LogLevel, message: &str, metadata: Option<Value>) {
        let log = LogMessage {
            id: Uuid::new_v4(),
            level,
            message: message.to_string(),
            metadata: metadata.unwrap_or(serde_json::json!({})),
            timestamp: Utc::now().to_rfc3339(),
        };
        self.queue.push(log);
        self.notify.notify_one();
    }

    // Convenience methods for different log levels
    pub fn debug(&self, message: &str, metadata: Option<Value>) {
        self.log(LogLevel::DEBUG, message, metadata);
    }

    pub fn info(&self, message: &str, metadata: Option<Value>) {
        self.log(LogLevel::INFO, message, metadata);
    }

    pub fn warn(&self, message: &str, metadata: Option<Value>) {
        self.log(LogLevel::WARN, message, metadata);
    }

    pub fn error(&self, message: &str, metadata: Option<Value>) {
        self.log(LogLevel::ERROR, message, metadata);
    }

    pub fn fatal(&self, message: &str, metadata: Option<Value>) {
        self.log(LogLevel::FATAL, message, metadata);
    }
}
