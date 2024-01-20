use chrono::{DateTime, FixedOffset};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: Option<String>,
    pub first_date: Option<DateTime<FixedOffset>>,
    pub last_date: Option<DateTime<FixedOffset>>,
    pub title_medium: Option<ObjectId>,
    pub media: Vec<ObjectId>,
}