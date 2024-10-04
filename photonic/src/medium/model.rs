use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct MediumTag {
    pub medium_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "medium_type_enum")]
#[sqlx(rename_all = "lowercase")]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Medium {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub album_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub(super) struct MediumDb {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub medium_type: MediumType,
    pub album_id: Option<Uuid>,
    pub deleted_at: Option<NaiveDateTime>,
}
