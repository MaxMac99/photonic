use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{model::MediumItem, store::Store};

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
        self.config.storage.tmp_path.join(format!(
            "{}.{}",
            Uuid::new_v4().as_hyphenated(),
            ext
        ))
    }

    pub fn get_full_path(&self, medium_item: &MediumItem) -> PathBuf {
        if medium_item.original_store {
            self.config.storage.base_path.join(&medium_item.path)
        } else {
            self.config.storage.cache_path.join(&medium_item.path)
        }
    }
}
