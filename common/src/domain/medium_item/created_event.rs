use crate::{
    medium_item::MediumItemType,
    stream::events::{avro_serializations, Event, StorageLocation, Topic},
};
use avro_reference::{utils::TimestampMillis, AvroReferenceSchema};
use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, Utc};
use derive_builder::Builder;
use mime_serde_shim::Wrapper as Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, AvroReferenceSchema)]
#[avro(referencable, namespace = "de.vissing.photonic")]
pub struct MediumItemCreatedEvent {
    #[serde(skip)]
    #[avro(skip)]
    pub id: Uuid,
    pub medium_id: Uuid,
    #[avro(reference)]
    pub medium_item_type: MediumItemType,
    #[avro(reference)]
    pub location: StorageLocation,
    #[serde(with = "avro_serializations::byte")]
    #[avro(replace_type = "i64")]
    pub size: Byte,
    #[avro(replace_type = "String")]
    pub mime: Mime,
    pub filename: String,
    pub extension: String,
    pub user: Uuid,
    pub priority: i32,
    #[avro(replace_type = "Option<String>")]
    pub date_taken: Option<DateTime<FixedOffset>>,
    #[serde(with = "avro_serializations::date_time_utc")]
    #[avro(replace_type = "TimestampMillis")]
    pub date_added: DateTime<Utc>,
}

impl Event for MediumItemCreatedEvent {
    fn topic() -> Topic {
        Topic::MediumItemCreated
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn store_id(&mut self, id: &String) {
        self.id = Uuid::parse_str(id.as_str()).unwrap();
    }
}
