mod avro_serializations;
mod common;
mod medium_item_created;
mod medium_item_exif_loaded;

pub use common::{StorageLocation, StorageVariant};
pub use medium_item_created::*;
pub use medium_item_exif_loaded::*;

use avro_reference::{AvroReferenceSchema, ReferenceSchema};
use strum::Display;

#[derive(Display, PartialEq, Hash, Eq)]
pub enum Topic {
    MediumItemCreated,
    MediumItemExifLoaded,
}

impl Topic {
    pub fn subject_name(&self) -> String {
        self.to_string()
    }

    pub fn schema(&self) -> ReferenceSchema {
        match self {
            Topic::MediumItemCreated => MediumItemCreatedEvent::get_reference_schema(),
            Topic::MediumItemExifLoaded => MediumItemExifLoadedEvent::get_reference_schema(),
        }
    }
}

pub trait Event {
    fn topic() -> Topic;
    fn id(&self) -> String;
    fn store_id(&mut self, id: &String);
}
