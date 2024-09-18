use super::LogHandler;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Custom error type for MemoryHandler.
#[derive(Error, Debug)]
pub enum MemoryHandlerError {
    #[error("Mutex lock error: {0}")]
    LockError(String),
}

/// Handles in-memory logging with a fixed capacity.
pub struct MemoryHandler {
    buffer: Arc<Mutex<VecDeque<String>>>,
    capacity: usize,
}

impl MemoryHandler {
    /// Initializes the MemoryHandler with a specific capacity.
    pub fn new(capacity: usize) -> Self {
        MemoryHandler {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// Retrieves a copy of the current logs in memory.
    pub async fn get_logs(&self) -> Vec<String> {
        let buf = self.buffer.lock().await;
        buf.iter().cloned().collect()
    }
}

#[async_trait]
impl LogHandler for MemoryHandler {
    async fn emit(&self, formatted: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = self.buffer.lock().await;
        if buf.len() == self.capacity {
            buf.pop_front();
        }
        buf.push_back(formatted.to_string());
        Ok(())
    }
}
