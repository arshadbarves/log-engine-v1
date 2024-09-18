use super::LogHandler;
use async_trait::async_trait;
use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

/// Custom error type for FileHandler.
#[derive(Error, Debug)]
pub enum FileHandlerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Compression error: {0}")]
    CompressionError(String),
}

/// Handles file system logging with rotation and compression.
pub struct FileHandler {
    file_path: PathBuf,
    max_size: u64, // in bytes
    current_size: Arc<Mutex<u64>>,
}

impl FileHandler {
    /// Initializes the FileHandler with a file path and maximum file size for rotation.
    pub fn new(file_path: PathBuf, max_size: u64) -> Self {
        FileHandler {
            file_path,
            max_size,
            current_size: Arc::new(Mutex::new(0)),
        }
    }

    /// Checks if log rotation is needed and performs it.
    async fn rotate_if_needed(&self) -> Result<(), FileHandlerError> {
        let mut size = self.current_size.lock().await;
        if *size >= self.max_size {
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let rotated_name = format!("{}.{}", self.file_path.display(), timestamp);
            tokio::fs::rename(&self.file_path, rotated_name.clone()).await?;

            // Compress the rotated file
            let rotated_path = PathBuf::from(rotated_name.clone());
            let compressed_path = rotated_path.with_extension("gz");
            let mut original = File::open(&rotated_path).await?;
            let mut content = Vec::new();
            original.read_to_end(&mut content).await?;

            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(&content)
                .map_err(|e| FileHandlerError::CompressionError(e.to_string()))?;
            let compressed_data = encoder
                .finish()
                .map_err(|e| FileHandlerError::CompressionError(e.to_string()))?;

            tokio::fs::write(&compressed_path, compressed_data).await?;
            tokio::fs::remove_file(&rotated_path).await?;

            *size = 0;
        }
        Ok(())
    }
}

#[async_trait]
impl LogHandler for FileHandler {
    async fn emit(&self, formatted: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.rotate_if_needed().await?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await?;

        let bytes = formatted.as_bytes();
        file.write_all(bytes).await?;
        file.write_all(b"\n").await?;

        let mut size = self.current_size.lock().await;
        *size += bytes.len() as u64 + 1; // +1 for newline
        Ok(())
    }
}
