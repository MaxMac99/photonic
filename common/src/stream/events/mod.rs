mod avro_serializations;
mod common;
mod medium_item_created;
mod medium_item_exif_loaded;

pub use common::{StorageLocation, StorageVariant};
pub use medium_item_created::*;
pub use medium_item_exif_loaded::*;

use apache_avro::Schema;
use lazy_static::lazy_static;
use strum::Display;

lazy_static! {
    static ref SCHEMAS: Vec<Schema> = Schema::parse_list(&vec![
        include_str!("../../../resources/schemas/common/date_time_tz.avsc"),
        include_str!("../../../resources/schemas/common/storage_location.avsc"),
        include_str!("../../../resources/schemas/medium_item_created.avsc"),
        include_str!("../../../resources/schemas/medium_item_exif_loaded.avsc"),
    ])
    .unwrap();
    pub static ref COMMON_DATE_TIME_TZ_SCHEMA: &'static Schema = &SCHEMAS[0];
    pub static ref COMMON_STORAGE_LOCATION_SCHEMA: &'static Schema = &SCHEMAS[1];
    pub static ref MEDIUM_ITEM_CREATED_SCHEMA: &'static Schema = &SCHEMAS[2];
    pub static ref MEDIUM_ITEM_EXIF_LOADED_SCHEMA: &'static Schema = &SCHEMAS[3];
    pub static ref COMMON_SCHEMATA: Vec<&'static Schema> =
        vec![*COMMON_DATE_TIME_TZ_SCHEMA, *COMMON_STORAGE_LOCATION_SCHEMA];
}

#[derive(Display, PartialEq, Hash, Eq)]
pub enum Topic {
    MediumItemCreated,
    MediumItemExifLoaded,
}

impl Topic {
    pub fn subject_name(&self) -> String {
        self.to_string()
    }

    pub fn schema(&self) -> &Schema {
        match self {
            Topic::MediumItemCreated => *MEDIUM_ITEM_CREATED_SCHEMA,
            Topic::MediumItemExifLoaded => *MEDIUM_ITEM_EXIF_LOADED_SCHEMA,
        }
    }
}

pub trait Event {
    fn topic() -> Topic;
    fn get_schema() -> &'static Schema;
    fn id(&self) -> String;
    fn store_id(&mut self, id: &String);
}
