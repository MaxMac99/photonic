use std::{collections::HashMap, fmt::Debug, path::Path, str::FromStr};

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use mime::Mime;
use serde_json::Value;
use snafu::OptionExt;

use exiftool::{Exiftool, Metadata};

use crate::{error::ExtractMimetypeSnafu, MetaError, MetaInfo};

#[derive(Debug)]
pub struct Service {
    exiftool: Exiftool,
}

impl Service {
    pub async fn new() -> Result<Self, MetaError> {
        Ok(Self {
            exiftool: Exiftool::new().await?,
        })
    }

    pub async fn read_file<P>(
        &self,
        path: P,
        with_additional_data: bool,
    ) -> Result<MetaInfo, MetaError>
    where
        P: AsRef<Path> + Debug,
    {
        let metadata = self
            .exiftool
            .read_file(path, with_additional_data, false)
            .await?;

        let camera_make = metadata
            .get("Make")
            .and_then(|val| val.value.as_str())
            .map(|val| String::from(val));
        let camera_model = metadata
            .get("Model")
            .and_then(|val| val.value.as_str())
            .map(|val| String::from(val));
        let mimetype: Mime = metadata
            .get("MIMEType")
            .and_then(|value| value.value.as_str())
            .and_then(|value| Mime::from_str(value).ok())
            .context(ExtractMimetypeSnafu)?;

        let date = Self::extract_date_time(&metadata);
        let mut additional_data = HashMap::new();
        if with_additional_data {
            let parsed_values: Vec<String> = vec![
                "Make",
                "Model",
                "MIMEType",
                "SubSecDateTimeOriginal",
                "DateTimeOriginal",
            ]
            .into_iter()
            .map(String::from)
            .collect();
            additional_data = metadata
                .into_iter()
                .filter(|(key, _)| !parsed_values.contains(key))
                .flat_map(|(key, value)| match value.value {
                    Value::Null => None,
                    Value::String(val) => Some((key, val)),
                    _ => Some((key, format!("{}", value.value))),
                })
                .collect();
        }
        let meta_info = MetaInfo {
            date,
            camera_make,
            camera_model,
            mimetype,
            additional_data,
        };
        Ok(meta_info)
    }

    fn extract_date_time(
        metadata: &HashMap<String, Metadata>,
    ) -> Option<DateTime<FixedOffset>> {
        metadata
            .get("SubSecDateTimeOriginal")
            .or_else(|| metadata.get("DateTimeOriginal"))
            .and_then(|value| value.value.as_str())
            .and_then(Self::parse_date_time)
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
}

#[cfg(test)]
mod test {
    use chrono::{FixedOffset, NaiveDate, TimeZone};

    use crate::Service;

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
        assert_eq!(Service::parse_date_time(date_str), Some(expected));
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
        assert_eq!(Service::parse_date_time(date_str), Some(expected));
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
        assert_eq!(Service::parse_date_time(date_str), Some(expected));
    }
}
