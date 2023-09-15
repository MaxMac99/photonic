use chrono::{DateTime, FixedOffset};
use mime::Mime;
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Clone)]
pub struct CreateMediumInput {
    pub album_id: Option<ObjectId>,
    pub filename: String,
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub mime: Mime,
}