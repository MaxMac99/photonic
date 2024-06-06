use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    model::{FileItem, StoreLocation},
    store::Store,
};

#[derive(Debug, Default, Clone)]
pub struct PathOptions {
    pub username: String,
    pub album: Option<String>,
    pub album_year: Option<u32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date: DateTime<Utc>,
    pub timezone: i32,
    pub filename: String,
    pub extension: String,
}

impl Store {
    pub fn get_temp_file_path(&self, ext: &str) -> PathBuf {
        self.config
            .storage
            .tmp_path
            .join(format!("{}.{}", Uuid::new_v4().as_hyphenated(), ext))
    }

    pub fn get_full_path(&self, file_item: &FileItem) -> PathBuf {
        match file_item.location {
            StoreLocation::Originals => self.config.storage.base_path.join(&file_item.path),
            StoreLocation::Cache => self.config.storage.cache_path.join(&file_item.path),
        }
    }

    pub fn get_full_path_from_relative(&self, location: StoreLocation, path: &PathBuf) -> PathBuf {
        match location {
            StoreLocation::Originals => self.config.storage.base_path.join(path),
            StoreLocation::Cache => self.config.storage.cache_path.join(path),
        }
    }
}
