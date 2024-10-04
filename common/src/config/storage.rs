use confique::Config;
use snafu::{ResultExt, Whatever};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Config)]
pub struct StorageConfig {
    #[config(default = "/storage", env = "STORAGE_BASE_DIRECTORY")]
    pub base_path: PathBuf,
    #[config(
        default = "/<album_year>/<album>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>"
    )]
    pub pattern: String,
    #[config(default = "/cache/store", env = "STORAGE_CACHE_DIRECTORY")]
    pub cache_path: PathBuf,
    #[config(default = "/cache/tmp", env = "STORAGE_TEMP_DIRECTORY")]
    pub tmp_path: PathBuf,
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
