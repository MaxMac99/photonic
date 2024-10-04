use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{common::StoreLocation, Config};

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

pub fn get_temp_file_path(config: &Arc<Config>, ext: &str) -> PathBuf {
    config
        .storage
        .tmp_path
        .join(format!("{}.{}", Uuid::new_v4().as_hyphenated(), ext))
}

pub fn get_full_path_from_relative(
    config: &Arc<Config>,
    location: StoreLocation,
    path: &PathBuf,
) -> PathBuf {
    match location {
        StoreLocation::Originals => config.storage.base_path.join(path),
        StoreLocation::Cache => config.storage.cache_path.join(path),
    }
}
