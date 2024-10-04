use super::{avro_serializations, Event, Topic};
use crate::{medium_item::MediumItemType, stream::events::common::StorageLocation};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::events::{StorageVariant, SCHEMAS};
    use apache_avro::{from_value, to_value, Codec, Reader, Writer};
    use chrono::NaiveDate;
    use std::{path::PathBuf, str::FromStr};

    #[test]
    fn can_get_schema() {
        MediumItemCreatedEvent::get_schema();
    }

    #[test]
    fn write_schema() {
        let event = new_file_created_event();

        let schema = MediumItemCreatedEvent::get_schema();
        let schemata: Vec<&Schema> = SCHEMAS.iter().collect();
        let mut writer = Writer::with_schemata(&schema, schemata, Vec::new(), Codec::Null);

        let value = to_value(event.clone()).unwrap();
        println!("{:?}", value);

        writer.append_ser(event).unwrap();
        writer.into_inner().unwrap();
    }

    #[test]
    fn read_schema() {
        let mut event = new_file_created_event();

        let schema = MediumItemCreatedEvent::get_schema();
        let schemata: Vec<&Schema> = SCHEMAS.iter().collect();
        let mut writer = Writer::with_schemata(&schema, schemata.clone(), Vec::new(), Codec::Null);
        writer.append_ser(event.clone()).unwrap();
        let encoded = writer.into_inner().unwrap();

        let mut reader = Reader::with_schemata(&schema, schemata, &encoded[..]).unwrap();
        let actual =
            from_value::<MediumItemCreatedEvent>(&reader.next().unwrap().unwrap()).unwrap();

        // Ignore id
        event.id = actual.id;
        assert_eq!(actual, event);
    }

    fn new_file_created_event() -> MediumItemCreatedEvent {
        MediumItemCreatedEvent {
            id: Uuid::new_v4(),
            medium_id: Uuid::new_v4(),
            location: StorageLocation {
                variant: StorageVariant::Temp,
                path: PathBuf::from("some/path"),
            },
            size: Byte::from_u64(1234567890),
            mime: Mime::from_str("application/json").unwrap(),
            filename: "hello".to_string(),
            extension: "txt".to_string(),
            user: Uuid::new_v4(),
            priority: 123,
            date_taken: Some(
                NaiveDate::from_ymd_opt(2023, 08, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 40, 825)
                    .unwrap()
                    .and_local_timezone(FixedOffset::east_opt(2 * 60 * 60).unwrap())
                    .earliest()
                    .unwrap(),
            ),
            date_added: NaiveDate::from_ymd_opt(2023, 08, 17)
                .unwrap()
                .and_hms_milli_opt(8, 58, 42, 825)
                .unwrap()
                .and_utc(),
        }
    }
}
