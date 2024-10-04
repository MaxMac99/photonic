use super::{Event, Topic};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::events::SCHEMAS;
    use apache_avro::{from_value, Codec, Reader, Writer};
    use chrono::NaiveDate;
    use std::str::FromStr;

    #[test]
    fn can_get_schema() {
        MediumItemExifLoadedEvent::get_schema();
    }

    #[test]
    fn write_schema() {
        let event = new_exif_loaded_event();

        let schema = MediumItemExifLoadedEvent::get_schema();
        let schemata: Vec<&Schema> = SCHEMAS.iter().collect();
        let mut writer = Writer::with_schemata(&schema, schemata, Vec::new(), Codec::Null);
        writer.append_ser(event).unwrap();
        writer.into_inner().unwrap();
    }

    #[test]
    fn read_schema() {
        let mut event = new_exif_loaded_event();

        let schema = MediumItemExifLoadedEvent::get_schema();
        let schemata: Vec<&Schema> = SCHEMAS.iter().collect();
        let mut writer = Writer::with_schemata(&schema, schemata.clone(), Vec::new(), Codec::Null);
        writer.append_ser(event.clone()).unwrap();
        let encoded = writer.into_inner().unwrap();

        let mut reader = Reader::with_schemata(&schema, schemata.clone(), &encoded[..]).unwrap();
        let actual =
            from_value::<MediumItemExifLoadedEvent>(&reader.next().unwrap().unwrap()).unwrap();

        // Ignore id
        event.id = actual.id;
        assert_eq!(actual, event);
    }

    fn new_exif_loaded_event() -> MediumItemExifLoadedEvent {
        MediumItemExifLoadedEvent {
            id: Uuid::new_v4(),
            date: Some(
                NaiveDate::from_ymd_opt(2023, 08, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 40, 825)
                    .unwrap()
                    .and_local_timezone(FixedOffset::east_opt(2 * 60 * 60).unwrap())
                    .earliest()
                    .unwrap(),
            ),
            mime: Some(Mime::from_str("application/json").unwrap()),
            camera_make: Some("Camera".to_string()),
            camera_model: Some("Model".to_string()),
            height: Some(12),
            width: Some(13),
        }
    }
}
