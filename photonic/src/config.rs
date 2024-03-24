use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};
use tokio::fs;

const ENV_STORAGE_BASE_DIRECTORY: &str = "STORAGE_BASE_DIRECTORY";
const ENV_STORAGE_PATTERN: &str = "STORAGE_PATTERN";
const ENV_STORAGE_CACHE_DIRECTORY: &str = "STORAGE_CACHE_DIRECTORY";

const ENV_DATABASE_URL: &str = "DATABASE_URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: Storage,
    pub database: Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub base_path: PathBuf,
    pub pattern: String,
    pub cache_path: PathBuf,
    pub tmp_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
}

const DEFAULT_STORAGE_BASE_DIRECTORY: &str = "/storage";
const DEFAULT_STORAGE_PATTERN: &str =
    "/<album_year>/<album>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>";
const DEFAULT_STORAGE_CACHE_DIRECTORY: &str = "/cache";

impl Config {
    pub async fn load() -> Result<Self, Whatever> {
        let storage = Config::create_storage().await?;
        let database = Config::create_database()?;

        let config = Self { storage, database };

        Ok(config)
    }

    async fn create_storage() -> Result<Storage, Whatever> {
        let base_path = Self::get_or_create_directory(
            ENV_STORAGE_BASE_DIRECTORY,
            DEFAULT_STORAGE_BASE_DIRECTORY,
        )
        .await?;
        let pattern =
            std::env::var(ENV_STORAGE_PATTERN).unwrap_or(String::from(DEFAULT_STORAGE_PATTERN));
        let cache_path = Self::get_or_create_directory(
            ENV_STORAGE_CACHE_DIRECTORY,
            DEFAULT_STORAGE_CACHE_DIRECTORY,
        )
        .await?;
        let tmp_path =
            Self::get_or_create_directory("", cache_path.join("tmp").to_str().unwrap()).await?;

        Ok(Storage {
            base_path,
            pattern,
            cache_path,
            tmp_path,
        })
    }

    fn create_database() -> Result<Database, Whatever> {
        let url = std::env::var(ENV_DATABASE_URL).whatever_context("Could not get database url")?;

        Ok(Database { url })
    }

    async fn get_or_create_directory(env_var: &str, default: &str) -> Result<PathBuf, Whatever> {
        let mut base_path = std::env::var(env_var)
            .map(|val| PathBuf::from(val))
            .unwrap_or(PathBuf::from(default));

        if !base_path.starts_with("/") {
            let cwd =
                std::env::current_dir().whatever_context("Could not get current directory")?;
            base_path = cwd.join(base_path);
        }

        fs::create_dir_all(&base_path)
            .await
            .with_whatever_context(|_| {
                format!(
                    "Could not create directory at path {:?} fo env var {}",
                    base_path.clone(),
                    env_var
                )
            })?;
        fs::canonicalize(&base_path)
            .await
            .with_whatever_context(|_| {
                format!(
                    "Could not canonicalize directory at path {:?} fo env var {}",
                    base_path.clone(),
                    env_var
                )
            })
    }
}
