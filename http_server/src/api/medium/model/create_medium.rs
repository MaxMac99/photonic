use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

use core::ObjectId;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMediumInput {
    pub album_id: Option<ObjectId>,
    pub filename: String,
    pub extension: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
}
