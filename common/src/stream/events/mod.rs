pub(crate) mod avro_serializations;
mod common;

pub use common::{StorageLocation, StorageVariant};

use avro_reference::AvroReferenceSchema;

pub trait Event {
    fn topic() -> &'static str;
    fn id(&self) -> String;
    fn store_id(&mut self, id: &String);
}
