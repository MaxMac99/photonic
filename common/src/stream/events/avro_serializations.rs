use serde::{Deserialize, Deserializer, Serializer};

pub(crate) mod byte {
    use super::*;
    use byte_unit::Byte;

    pub fn serialize<S>(x: &Byte, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_i64(x.as_u64() as i64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Byte, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: i64 = Deserialize::deserialize(deserializer)?;
        Ok(Byte::from_u64(val as u64))
    }
}

pub(crate) mod date_time_utc {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};

    pub fn serialize<S>(x: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_i64(x.timestamp_micros())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = i64::deserialize(deserializer)?;
        Ok(Utc.timestamp_micros(val).unwrap())
    }
}
