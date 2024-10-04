use common::config::{ServerConfig, StorageConfig, StreamConfig};
use confique::Config;
use snafu::{ResultExt, Whatever};
use tracing::log::debug;

#[derive(Debug, Config)]
pub struct StorageWorkerConfig {
    #[config(nested)]
    pub stream: StreamConfig,
    #[config(nested)]
    pub server: ServerConfig,
    #[config(nested)]
    pub storage: StorageConfig,
}

impl StorageWorkerConfig {
    pub async fn load() -> Result<Self, Whatever> {
        let mut config = StorageWorkerConfig::builder()
            .env()
            .load()
            .whatever_context("Could not build config")?;
        config.storage.setup().await?;

        debug!("Config: {:?}", config);

        Ok(config)
    }
}
