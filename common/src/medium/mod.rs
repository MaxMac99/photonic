use avro_reference::AvroReferenceSchema;
use mime::Mime;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema, AvroReferenceSchema,
)]
#[sqlx(type_name = "medium_type_enum", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[avro(referencable)]
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
