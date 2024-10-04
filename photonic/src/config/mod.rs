use crate::config::{
    cache::Cache, database::DatabaseConfig, oauth::OAuth, storage::Storage, streams::Streams,
};
use serde::{Deserialize, Serialize};
use snafu::Whatever;

mod cache;
mod database;
mod oauth;
mod storage;
mod streams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: Storage,
    pub database: DatabaseConfig,
    pub oauth: OAuth,
    pub cache: Cache,
    pub streams: Streams,
}

impl Config {
    pub async fn load() -> Result<Self, Whatever> {
        Ok(Self {
            storage: Storage::load().await?,
            database: DatabaseConfig::load()?,
            oauth: OAuth::load()?,
            cache: Cache::load()?,
            streams: Streams::load()?,
        })
    }
}
