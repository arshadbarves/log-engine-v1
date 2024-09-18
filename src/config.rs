use config::{Config as ConfigLoader, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub filters: Option<HashMap<String, String>>,
    pub handlers: Vec<HandlerConfig>,
    pub formatter: Option<String>,
    pub plugins: Option<Vec<PluginConfig>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HandlerConfig {
    pub type_: String,
    pub level: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration load error: {0}")]
    LoadError(String),
}

#[derive(Clone)]
pub struct ConfigurationManager {
    config: Arc<RwLock<LogConfig>>,
}

impl ConfigurationManager {
    /// Initializes the ConfigurationManager with a configuration file.
    pub async fn new(config_file: &str) -> Result<Self, ConfigError> {
        if !Path::new(config_file).exists() {
            return Err(ConfigError::LoadError(format!("Configuration file not found: {}", config_file)));
        }

        let builder = ConfigLoader::builder()
            .add_source(File::with_name(config_file))
            .add_source(Environment::with_prefix("LOGENGINE"));

        let settings = builder
            .build()
            .map_err(|e| ConfigError::LoadError(e.to_string()))?;

        let config: LogConfig = settings
            .try_deserialize()
            .map_err(|e| ConfigError::LoadError(format!("Failed to parse configuration: {}", e)))?;

        Ok(ConfigurationManager {
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Retrieves the current configuration.
    pub async fn get_config(&self) -> LogConfig {
        self.config.read().await.clone()
    }

    /// Updates the current configuration.
    pub async fn update_config(&self, new_config: LogConfig) {
        let mut cfg = self.config.write().await;
        *cfg = new_config;
    }

    /// Watches the configuration file for changes and updates dynamically.
    pub async fn watch_config(&self, config_file: &str) -> Result<(), ConfigError> {
        // Implementation for watching the config file using tokio's file watcher or notify crate.
        // Placeholder for brevity.
        Ok(())
    }
}
