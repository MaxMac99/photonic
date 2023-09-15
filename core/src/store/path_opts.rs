use chrono::{DateTime, FixedOffset};

#[derive(Debug, Default)]
pub struct PathOptions {
    pub album: Option<String>,
    pub album_year: Option<u32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date: DateTime<FixedOffset>,
    pub filename: String,
    pub extension: String,
}
