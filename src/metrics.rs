use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Failed to bind to address: {0}")]
    BindError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub struct MetricsManager {
    pub logs_processed: Arc<AtomicUsize>,
    pub errors: Arc<AtomicUsize>,
    pub queue_size: Arc<AtomicUsize>,
}

impl MetricsManager {
    /// Initializes the MetricsManager.
    pub fn new() -> Self {
        MetricsManager {
            logs_processed: Arc::new(AtomicUsize::new(0)),
            errors: Arc::new(AtomicUsize::new(0)),
            queue_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Increments the log count counter.
    pub fn increment_log_count(&self) {
        self.logs_processed.fetch_add(1, Ordering::SeqCst);
    }

    /// Increments the error counter.
    pub fn increment_error(&self) {
        self.errors.fetch_add(1, Ordering::SeqCst);
    }

    /// Sets the current queue size gauge.
    pub fn set_queue_size(&self, size: usize) {
        self.queue_size.store(size, Ordering::SeqCst);
    }

    /// Starts an HTTP server to expose metrics.
    pub async fn serve_metrics(&self, addr: &str) -> Result<(), MetricsError> {
        let listener = TcpListener::bind(addr).await.map_err(|e| MetricsError::BindError(e.to_string()))?;
        println!("Metrics server running on {}", addr);

        loop {
            let (mut socket, _) = listener.accept().await.map_err(|e| MetricsError::IoError(e.to_string()))?;
            let logs_processed = self.logs_processed.clone();
            let errors = self.errors.clone();
            let queue_size = self.queue_size.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(&mut socket);
                let mut request = String::new();
                if reader.read_line(&mut request).await.is_ok() {
                    if request.starts_with("GET /metrics") {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n\
                            logs_processed {}\nerrors {}\nqueue_size {}\n",
                            logs_processed.load(Ordering::SeqCst),
                            errors.load(Ordering::SeqCst),
                            queue_size.load(Ordering::SeqCst),
                        );
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                }
            });
        }
    }
}
