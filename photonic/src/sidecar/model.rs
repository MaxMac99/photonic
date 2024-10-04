use crate::common::StoreLocation;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Sidecar {
    pub id: Uuid,
    pub medium_id: Uuid,
    pub mime: String,
    pub filename: String,
    pub path: String,
    pub filesize: i64,
    pub location: StoreLocation,
    pub priority: i32,
    pub last_saved: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}
