use confique::Config;
use snafu::{ResultExt, Whatever};
use tracing::log::debug;

mod database;
mod server;
mod storage;

pub use database::DatabaseConfig;
pub use server::ServerConfig;
pub use storage::StorageConfig;

#[derive(Debug, Config)]
pub struct GlobalConfig {
    #[config(nested)]
    pub server: ServerConfig,
    #[config(nested)]
    pub storage: StorageConfig,
    #[config(nested)]
    pub database: DatabaseConfig,
}

impl GlobalConfig {
    pub async fn load() -> Result<Self, Whatever> {
        let mut config = GlobalConfig::builder()
            .env()
            .load()
            .whatever_context("Could not build config")?;
        config.storage.setup().await?;

        debug!("Config: {:?}", config);

        Ok(config)
    }
}
