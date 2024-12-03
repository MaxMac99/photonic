use crate::{
    medium::MediumType,
    stream::events::{avro_serializations, Event},
};
use avro_reference::{utils::TimestampMillis, AvroReferenceSchema};
use chrono::{DateTime, FixedOffset, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, AvroReferenceSchema)]
#[avro(referencable, namespace = "de.vissing.photonic")]
pub struct MediumUpdatedEvent {
    #[serde(skip)]
    #[avro(skip)]
    pub id: Uuid,
    #[avro(reference)]
    pub medium_type: MediumType,
    #[avro(replace_type = "Option<String>")]
    pub date_taken: Option<DateTime<FixedOffset>>,
    #[serde(with = "avro_serializations::date_time_utc")]
    #[avro(replace_type = "TimestampMillis")]
    pub date_added: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl Event for MediumUpdatedEvent {
    fn topic() -> String {
        "MediumUpdated"
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn store_id(&mut self, id: &String) {
        self.id = Uuid::parse_str(id.as_str()).unwrap();
    }
}
