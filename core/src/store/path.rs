use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::store::Store;

#[derive(Debug, Default)]
pub struct PathOptions {
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
        self.config.storage.cache_path.join("tmp").join(format!("{}.{}", Uuid::new_v4().as_hyphenated(), ext))
    }
}
