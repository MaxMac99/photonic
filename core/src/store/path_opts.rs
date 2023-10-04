use chrono::{DateTime, Utc};

#[derive(Debug, Default)]
pub struct PathOptions {
    pub album: Option<String>,
    pub album_year: Option<u32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date: DateTime<Utc>,
    pub filename: String,
    pub extension: String,
}
