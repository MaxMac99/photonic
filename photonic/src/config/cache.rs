use redis::{ConnectionInfo, IntoConnectionInfo};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};
use std::path::PathBuf;

const ENV_CACHE_CONNECTION: &str = "CACHE_CONNECTION";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    pub redis_connection: ConnectionInfo,
}

impl Cache {
    pub(crate) fn load() -> Result<Self, Whatever> {
        let redis_connection = std::env::var(ENV_CACHE_CONNECTION)
            .whatever_context(format!("Could not find env {}", ENV_CACHE_CONNECTION))?
            .into_connection_info()
            .whatever_context("Invalid connection info to redis")?;

        Ok(Cache { redis_connection })
    }
}
