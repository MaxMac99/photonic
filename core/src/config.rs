use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::Error;

const ENV_STORAGE_BASE_DIRECTORY: &str = "STORAGE_BASE_DIRECTORY";
const ENV_STORAGE_PATTERN: &str = "STORAGE_PATTERN";

const ENV_MONGO_URL: &str = "MONGO_URL";
const ENV_MONGO_USERNAME: &str = "MONGO_USER";
const ENV_MONGO_PASSWORD: &str = "MONGO_PASSWORD";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: Storage,
    pub mongo: Mongo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub base_path: PathBuf,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mongo {
    pub url: String,
    pub username: String,
    pub password: String,
}

const DEFAULT_STORAGE_BASE_DIRECTORY: &str = "/storage";
const DEFAULT_STORAGE_PATTERN: &str = "/<album_year>/<album>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>";

impl Config {
    pub async fn load() -> Result<Self, Error> {
        let storage = Config::create_storage().await?;
        let mongo = Config::create_mongo()?;

        let config = Self {
            storage,
            mongo,
        };

        Ok(config)
    }

    async fn create_storage() -> Result<Storage, Error> {
        let mut base_path = std::env::var(ENV_STORAGE_BASE_DIRECTORY)
            .map(|val| PathBuf::from(val))
            .unwrap_or(PathBuf::from(DEFAULT_STORAGE_BASE_DIRECTORY));

        if !base_path.starts_with("/") {
            let cwd = std::env::current_dir()
                .map_err(|err| Error::Internal(format!("Could not find current working directory: {}", err.to_string())))?;
            base_path = cwd.join(base_path);
        }

        fs::create_dir_all(&base_path)
            .await
            .map_err(|err| Error::Internal(format!("Could not create directories for path {:?}: {}", &base_path, err.to_string())))?;
        let canonicalized = fs::canonicalize(&base_path)
            .await
            .map_err(|err| Error::Internal(format!("Could not canonicalize path {:?}: {}", &base_path, err.to_string())))?;

        let pattern = std::env::var(ENV_STORAGE_PATTERN)
            .unwrap_or(String::from(DEFAULT_STORAGE_PATTERN));

        Ok(Storage {
            base_path: canonicalized,
            pattern,
        })
    }

    fn create_mongo() -> Result<Mongo, Error> {
        let url = std::env::var(ENV_MONGO_URL)
            .map_err(|err| Error::Internal(format!("Could not find mongodb url: {}", err.to_string())))?;
        let username = std::env::var(ENV_MONGO_USERNAME)
            .map_err(|err| Error::Internal(format!("Could not find mongodb user: {}", err.to_string())))?;
        let password = std::env::var(ENV_MONGO_PASSWORD)
            .map_err(|err| Error::Internal(format!("Could not find mongodb password: {}", err.to_string())))?;

        Ok(Mongo {
            url,
            username,
            password,
        })
    }
}
