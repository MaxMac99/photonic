use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: Uuid,
    pub owner: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "firstDate", skip_serializing_if = "Option::is_none")]
    pub first_date: Option<DateTime<FixedOffset>>,
    #[serde(rename = "lastDate", skip_serializing_if = "Option::is_none")]
    pub last_date: Option<DateTime<FixedOffset>>,
    #[serde(rename = "titleMedium", skip_serializing_if = "Option::is_none")]
    pub title_medium: Option<Uuid>,
    pub media: Vec<Uuid>,
}
