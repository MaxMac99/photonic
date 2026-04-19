pub mod mime_serde {
    use mime::Mime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(mime: &Mime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(mime.as_ref())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Mime>().map_err(serde::de::Error::custom)
    }
}

pub mod option_mime_serde {
    use mime::Mime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(mime: &Option<Mime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match mime {
            Some(m) => serializer.serialize_some(m.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Mime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => s
                .parse::<Mime>()
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}
