use chrono::{DateTime, Utc};
use serde::Deserialize;

use fotonic::ObjectId;

#[derive(Debug, Clone, Deserialize)]
pub struct FindAllMediumInput {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub album_id: Option<ObjectId>,
    pub include_no_album: Option<bool>,
}
