use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;

use core::ObjectId;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMediumInput {
    pub album_id: Option<ObjectId>,
    pub filename: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FindAllMediumInput {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub album_id: Option<ObjectId>,
    pub include_no_album: Option<bool>,
}
