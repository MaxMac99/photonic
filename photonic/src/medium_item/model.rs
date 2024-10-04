use crate::common::StoreLocation;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
pub enum MediumItemType {
    Original,
    Edit,
    Preview,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct MediumItem {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub medium_item_type: MediumItemType,
    pub mime: String,
    pub filename: String,
    pub path: String,
    #[sqlx(try_from = "i64")]
    pub filesize: u64,
    pub location: StoreLocation,
    pub priority: i32,
    pub timezone: i32,
    pub taken_at: NaiveDateTime,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub width: i32,
    pub height: i32,
}
