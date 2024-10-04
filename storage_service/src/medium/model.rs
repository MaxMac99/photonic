use chrono::{DateTime, FixedOffset};

use mime::Mime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "medium_type_enum")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

impl From<Mime> for MediumType {
    fn from(value: Mime) -> Self {
        match (value.type_(), value.subtype()) {
            (mime::IMAGE, mime::SVG) => MediumType::Vector,
            (mime::IMAGE, mime::GIF) => MediumType::Gif,
            (mime::IMAGE, _) => MediumType::Photo,
            (mime::VIDEO, _) => MediumType::Video,
            _ => MediumType::Other,
        }
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct CreateMediumInput {
    pub filename: String,
    pub extension: String,
    #[serde(default = "default_prio")]
    pub priority: i32,
    #[serde(default)]
    pub tags: Vec<String>,
    pub medium_type: Option<MediumType>,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

fn default_prio() -> i32 {
    10
}
