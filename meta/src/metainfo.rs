use std::{collections::HashMap, io::Cursor};

use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use exif::{Exif, Field, In, Reader, Tag, Value};
use mime::Mime;
use snafu::OptionExt;

use crate::{error::ExtractMimetypeSnafu, MetaError};

#[derive(Debug)]
pub struct MetaInfo {
    pub date: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub mimetype: Mime,
    pub additional_data: HashMap<String, String>,
}

impl MetaInfo {
    pub fn from(data: &[u8], ext: &str) -> Result<Self, MetaError> {
        let lower_ext = ext.to_lowercase();
        let mimetype: Mime = infer::get(data)
            .and_then(|mime| mime.mime_type().parse().ok())
            .or_else(|| mime_guess::from_ext(&lower_ext).first())
            .context(ExtractMimetypeSnafu)?;

        let mut cursor = Cursor::new(data);
        let exifreader = Reader::new();
        let exif = exifreader.read_from_container(&mut cursor)?;

        let date = date_time_original(&exif)
            .or_else(|| date_time_digitalized(&exif))
            .or_else(|| date_time(&exif));

        let make = exif
            .get_field(Tag::Make, In::PRIMARY)
            .and_then(|make| field_as_str(make));

        let model = exif
            .get_field(Tag::Model, In::PRIMARY)
            .and_then(|model| field_as_str(model));

        Ok(Self {
            date,
            camera_make: make,
            camera_model: model,
            mimetype,
            additional_data: HashMap::new(),
        })
    }
}

fn field_as_str(field: &Field) -> Option<String> {
    match field.value {
        Value::Ascii(ref bytes) => std::str::from_utf8(&bytes[0])
            .ok()
            .map(|val| String::from(val)),
        _ => None,
    }
}

fn date_time(exif: &Exif) -> Option<DateTime<FixedOffset>> {
    let original = exif.get_field(Tag::DateTime, In::PRIMARY)?;
    let offset = exif.get_field(Tag::OffsetTime, In::PRIMARY);
    let sub_sec = exif.get_field(Tag::SubSecTime, In::PRIMARY);
    field_to_date(original, offset, sub_sec)
}

fn date_time_original(exif: &Exif) -> Option<DateTime<FixedOffset>> {
    let original = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)?;
    let offset = exif.get_field(Tag::OffsetTimeOriginal, In::PRIMARY);
    let sub_sec = exif.get_field(Tag::SubSecTimeOriginal, In::PRIMARY);
    field_to_date(original, offset, sub_sec)
}

fn date_time_digitalized(exif: &Exif) -> Option<DateTime<FixedOffset>> {
    let original = exif.get_field(Tag::DateTimeDigitized, In::PRIMARY)?;
    let offset = exif.get_field(Tag::OffsetTimeDigitized, In::PRIMARY);
    let sub_sec = exif.get_field(Tag::SubSecTimeDigitized, In::PRIMARY);
    field_to_date(original, offset, sub_sec)
}

fn field_to_date(
    date: &Field,
    offset: Option<&Field>,
    sub_sec: Option<&Field>,
) -> Option<DateTime<FixedOffset>> {
    let Value::Ascii(ref date_str) = date.value else {
        return None;
    };
    let mut date = exif::DateTime::from_ascii(&date_str[0]).ok()?;

    if let Some(offset_field) = offset {
        if let Value::Ascii(ref offset_str) = offset_field.value {
            _ = date.parse_offset(&offset_str[0]);
        }
    }

    if let Some(sub_sec_field) = sub_sec {
        if let Value::Ascii(ref sub_sec_str) = sub_sec_field.value {
            _ = date.parse_subsec(&sub_sec_str[0]);
        }
    }

    exif_datetime_to_chrono(&date)
}

fn exif_datetime_to_chrono(
    exif: &exif::DateTime,
) -> Option<DateTime<FixedOffset>> {
    let chrono_date = NaiveDate::from_ymd_opt(
        exif.year as i32,
        exif.month as u32,
        exif.day as u32,
    )
    .or(None)?
    .and_hms_nano_opt(
        exif.hour as u32,
        exif.minute as u32,
        exif.second as u32,
        exif.nanosecond.unwrap_or(0),
    )
    .or(None)?;
    let tz = exif
        .offset
        .and_then(|offset| FixedOffset::east_opt(offset as i32 * 60))
        .unwrap_or(
            FixedOffset::east_opt(Local::now().offset().local_minus_utc())
                .unwrap(),
        );
    chrono_date.and_local_timezone(tz).earliest()
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Read};

    use chrono::{FixedOffset, NaiveDate};
    use exif::Error::Io;
    use mime::Mime;

    use crate::{metainfo::exif_datetime_to_chrono, MetaError, MetaInfo};

    #[test]
    fn exif_heic() -> Result<(), MetaError> {
        let file = std::fs::File::open("../test/IMG_4598.HEIC")
            .map_err(|err| Io(err))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).map_err(|err| Io(err))?;

        let info = MetaInfo::from(&buffer, "heic")?;
        println!("{:?}", info.date);
        assert_eq!(
            info.date,
            Some(
                NaiveDate::from_ymd_opt(2023, 08, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 40, 825)
                    .unwrap()
                    .and_local_timezone(
                        FixedOffset::east_opt(2 * 60 * 60).unwrap()
                    )
                    .earliest()
                    .unwrap()
            )
        );
        assert_eq!(info.camera_make, Some(String::from("Apple")));
        assert_eq!(info.camera_model, Some(String::from("iPhone 14 Pro")));
        assert_eq!(info.mimetype, "image/heif".parse::<Mime>().unwrap());
        Ok(())
    }

    #[test]
    fn exif_dng() -> Result<(), MetaError> {
        let file = std::fs::File::open("../test/IMG_4597.DNG")
            .map_err(|err| Io(err))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).map_err(|err| Io(err))?;

        let info = MetaInfo::from(&buffer, "heic")?;
        assert_eq!(
            info.date,
            Some(
                NaiveDate::from_ymd_opt(2023, 08, 16)
                    .unwrap()
                    .and_hms_milli_opt(8, 58, 15, 779)
                    .unwrap()
                    .and_local_timezone(
                        FixedOffset::east_opt(2 * 60 * 60).unwrap()
                    )
                    .earliest()
                    .unwrap()
            )
        );
        assert_eq!(info.camera_make, Some(String::from("Apple")));
        assert_eq!(info.camera_model, Some(String::from("iPhone 14 Pro")));
        assert_eq!(info.mimetype, "image/tiff".parse::<Mime>().unwrap());
        Ok(())
    }

    #[test]
    fn conv_datetime() {
        let datetime = exif::DateTime {
            year: 2023,
            month: 1,
            day: 2,
            hour: 3,
            minute: 4,
            second: 5,
            nanosecond: Some(10),
            offset: Some(-60),
        };

        let conv = exif_datetime_to_chrono(&datetime);
        assert_eq!(
            conv.unwrap().to_rfc3339(),
            "2023-01-02T03:04:05.000000010-01:00"
        )
    }
}
