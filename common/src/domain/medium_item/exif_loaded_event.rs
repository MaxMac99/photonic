use crate::stream::events::{Event, Topic};
use avro_reference::AvroReferenceSchema;
use chrono::{DateTime, FixedOffset};
use derive_builder::Builder;
use mime_serde_shim::Wrapper as Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq, AvroReferenceSchema)]
#[avro(referencable, namespace = "de.vissing.photonic")]
pub struct MediumItemExifLoadedEvent {
    #[serde(skip)]
    #[avro(skip)]
    pub id: Uuid,
    #[avro(replace_type = "Option<String>")]
    pub date: Option<DateTime<FixedOffset>>,
    #[avro(replace_type = "Option<String>")]
    pub mime: Option<Mime>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
}

impl Event for MediumItemExifLoadedEvent {
    fn topic() -> Topic {
        Topic::MediumItemExifLoaded
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn store_id(&mut self, id: &String) {
        self.id = Uuid::parse_str(id.as_str()).unwrap();
    }
}
