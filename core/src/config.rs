use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::Error;

const ENV_STORAGE_BASE_DIRECTORY: &str = "STORAGE_BASE_DIRECTORY";
const ENV_STORAGE_PATTERN: &str = "STORAGE_PATTERN";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub base_path: PathBuf,
    pub pattern: String,
}

const DEFAULT_STORAGE_BASE_DIRECTORY: &str = "/storage";
const DEFAULT_STORAGE_PATTERN: &str = "/<album_year>/<album>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>";

impl Config {
    pub async fn load() -> Result<Self, Error> {
        // Storage
        let storage = Config::create_storage().await?;
        let config = Self {
            storage
        };

        Ok(config)
    }

    async fn create_storage() -> Result<Storage, Error> {
        let base_path = std::env::var(ENV_STORAGE_BASE_DIRECTORY)
            .map(|val| PathBuf::from(val))
            .unwrap_or(PathBuf::from(DEFAULT_STORAGE_BASE_DIRECTORY));

        fs::create_dir_all(&base_path)
            .await
            .map_err(|_| Error::Internal(String::from("Could not create base path")))?;
        let canonicalized = fs::canonicalize(&base_path)
            .await
            .map_err(|_| Error::Internal(String::from("Could not create base path")))?;

        let pattern = std::env::var(ENV_STORAGE_PATTERN)
            .unwrap_or(String::from(DEFAULT_STORAGE_PATTERN));

        Ok(Storage {
            base_path: canonicalized,
            pattern,
        })
    }
}
