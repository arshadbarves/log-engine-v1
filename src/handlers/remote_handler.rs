use super::LogHandler;
use async_trait::async_trait;
use std::time::Duration;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Custom error type for RemoteHandler.
#[derive(Error, Debug)]
pub enum RemoteHandlerError {
    #[error("Failed to connect to remote server: {0}")]
    ConnectionError(String),
    #[error("Failed to send log: {0}")]
    SendError(String),
}

/// Handles remote logging by sending log messages to a centralized server.
pub struct RemoteHandler {
    address: String,
    port: u16,
    retries: usize,
}

impl RemoteHandler {
    /// Initializes the RemoteHandler with a server address, port, and retry count.
    pub fn new(address: String, port: u16, retries: Option<usize>) -> Self {
        RemoteHandler {
            address,
            port,
            retries: retries.unwrap_or(3),
        }
    }

    /// Attempts to send the log message with retries.
    async fn send_with_retries(&self, message: &str) -> Result<(), RemoteHandlerError> {
        let mut attempt = 0;
        while attempt < self.retries {
            match TcpStream::connect((&*self.address, self.port)).await {
                Ok(mut stream) => {
                    if let Err(_e) = stream.write_all(message.as_bytes()).await {
                        attempt += 1;
                        tokio::time::sleep(Duration::from_millis((100 * attempt) as u64)).await;
                        continue;
                    }
                    return Ok(());
                }
                Err(_) => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_millis((100 * attempt) as u64)).await;
                }
            }
        }
        Err(RemoteHandlerError::SendError("Max retries exceeded".into()))
    }
}

#[async_trait]
impl LogHandler for RemoteHandler {
    async fn emit(&self, formatted: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send_with_retries(formatted)
            .await
            .map_err(|e| Box::new(e) as _)
    }
}
