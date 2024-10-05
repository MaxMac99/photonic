use crate::{
    config::ExifWorkerConfig,
    exiftool::{Exiftool, Metadata},
};
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use common::{
    error::Result,
    medium_item::{MediumItemCreatedEvent, MediumItemExifLoadedEvent},
    stream::producer::KafkaProducer,
};
use mime_serde_shim::Wrapper as Mime;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use uuid::Uuid;

pub async fn handle_file_created(
    exiftool: Arc<Exiftool>,
    producer: KafkaProducer,
    message: MediumItemCreatedEvent,
    config: Arc<ExifWorkerConfig>,
) -> Result<()> {
    let result = exiftool
        .read_file(message.location.full_path(&config.storage), false, false)
        .await?;
    let event = parse_metadata(message.id, result);
    producer.produce(event).await?;

    Ok(())
}

fn parse_metadata(id: Uuid, metadata: HashMap<String, Metadata>) -> MediumItemExifLoadedEvent {
    let date = extract_date_time(&metadata);
    let mime = metadata
        .get("MIMEType")
        .and_then(|val| val.value.as_str())
        .and_then(|val| Mime::from_str(val).ok());
    let camera_make = metadata
        .get("Make")
        .and_then(|val| val.value.as_str())
        .map(String::from);
    let camera_model = metadata
        .get("Model")
        .and_then(|val| val.value.as_str())
        .map(String::from);
    let height = metadata
        .get("ExifImageHeight")
        .and_then(|val| val.value.as_i64());
    let width = metadata
        .get("ExifImageWidth")
        .and_then(|val| val.value.as_i64());
    MediumItemExifLoadedEvent {
        id,
        date,
        mime,
        camera_make,
        camera_model,
        height,
        width,
    }
}

fn extract_date_time(metadata: &HashMap<String, Metadata>) -> Option<DateTime<FixedOffset>> {
    metadata
        .get("SubSecDateTimeOriginal")
        .or_else(|| metadata.get("DateTimeOriginal"))
        .and_then(|value| value.value.as_str())
        .and_then(parse_date_time)
}

fn parse_date_time(value: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(value, "%Y:%m:%d %H:%M:%S%.f%:z")
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y:%m:%d %H:%M:%S")
                .ok()
                .and_then(|val| {
                    FixedOffset::east_opt(0)
                        .unwrap()
                        .from_local_datetime(&val)
                        .earliest()
                })
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{FixedOffset, NaiveDate, TimeZone};

    #[test]
    fn parse_date_full() {
        let date_str = "2023:08:16 08:58:15.779+02:00";
        let expected = FixedOffset::east_opt(2 * 60 * 60)
            .unwrap()
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2023, 8, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 15, 779)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(parse_date_time(date_str), Some(expected));
    }

    #[test]
    fn parse_date_without_nano() {
        let date_str = "2023:08:16 08:58:15+02:00";
        let expected = FixedOffset::east_opt(2 * 60 * 60)
            .unwrap()
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2023, 8, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 15, 0)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(parse_date_time(date_str), Some(expected));
    }

    #[test]
    fn parse_date_without_offset() {
        let date_str = "2023:08:16 08:58:15";
        let expected = FixedOffset::east_opt(0)
            .unwrap()
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2023, 8, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 15, 0)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(parse_date_time(date_str), Some(expected));
    }
}
