pub mod console_handler;
pub mod file_handler;
pub mod memory_handler;
pub mod remote_handler;

use async_trait::async_trait;

/// Trait defining the interface for log handlers.
#[async_trait]
pub trait LogHandler: Send + Sync {
    /// Emits a formatted log message to the handler's destination.
    async fn emit(&self, formatted: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub use console_handler::ConsoleHandler;
pub use file_handler::FileHandler;
pub use memory_handler::MemoryHandler;
pub use remote_handler::RemoteHandler;
