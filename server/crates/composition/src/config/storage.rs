use std::path::PathBuf;

use confique::Config;
use snafu::{ResultExt, Whatever};
use tokio::fs;

#[derive(Debug, Config)]
pub struct StorageConfig {
    #[config(default = "/storage", env = "STORAGE_BASE_DIRECTORY")]
    pub base_path: PathBuf,
    #[config(default = "/<year>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>")]
    pub pattern: String,
    #[config(default = "/cache/store", env = "STORAGE_CACHE_DIRECTORY")]
    pub cache_path: PathBuf,
    #[config(
        default = "/<type>/<album_year>/<album>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>"
    )]
    pub cache_pattern: String,
    #[config(default = "/cache/tmp", env = "STORAGE_TEMP_DIRECTORY")]
    pub tmp_path: PathBuf,
    /// Default quota for new users (in bytes) - 10 GB
    #[config(default = 10737418240_u64, env = "STORAGE_DEFAULT_USER_QUOTA")]
    pub default_user_quota: u64,
    /// Maximum quota any user can have (in bytes) - 1 TB
    #[config(default = 1099511627776_u64, env = "STORAGE_MAX_USER_QUOTA")]
    pub max_user_quota: u64,
    /// TTL for temporary files in seconds (default: 24 hours)
    #[config(default = 86400_u64, env = "STORAGE_TEMP_TTL_SECONDS")]
    pub temp_ttl_seconds: u64,
    /// Interval between cleanup sweeps in seconds (default: 1 hour)
    #[config(default = 3600_u64, env = "STORAGE_CLEANUP_INTERVAL_SECONDS")]
    pub cleanup_interval_seconds: u64,
}

impl StorageConfig {
    pub async fn setup(&mut self) -> Result<(), Whatever> {
        self.base_path = Self::get_or_create_directory(self.base_path.clone()).await?;
        self.cache_path = Self::get_or_create_directory(self.cache_path.clone()).await?;
        self.tmp_path = Self::get_or_create_directory(self.tmp_path.clone()).await?;

        Ok(())
    }

    async fn get_or_create_directory(path: PathBuf) -> Result<PathBuf, Whatever> {
        let mut path = path;
        if !path.starts_with("/") {
            let cwd =
                std::env::current_dir().whatever_context("Could not get current directory")?;
            path = cwd.join(path);
        }

        fs::create_dir_all(&path).await.with_whatever_context(|_| {
            format!("Could not create directory at path {:?}", path.clone())
        })?;
        fs::canonicalize(&path).await.with_whatever_context(|_| {
            format!(
                "Could not canonicalize directory at path {:?}",
                path.clone()
            )
        })
    }
}
