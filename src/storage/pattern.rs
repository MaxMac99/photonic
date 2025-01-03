use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};

pub struct PatternFields {
    pub filename: Option<String>,
    pub extension: String,
    pub user: Option<String>,
    pub date: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub album: Option<String>,
    pub album_year: Option<i32>,
}

pub fn create_path(pattern: &str, fields: PatternFields) -> String {
    let filename = fields.filename.unwrap_or("Unknown".to_string());
    let user = fields.user.unwrap_or("Unknown".to_string());
    let date = fields.date.unwrap_or_else(|| {
        FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(1970, 1, 1, 0, 0, 0)
            .unwrap()
    });
    let camera_make = fields.camera_make.unwrap_or("Unknown".to_string());
    let camera_model = fields.camera_model.unwrap_or("Unknown".to_string());
    let album = fields.album.unwrap_or("Unknown".to_string());
    let album_year = fields.album_year.unwrap_or(1970);

    let mut pattern = pattern
        .to_string()
        .replace("<filename>", &filename)
        .replace("<extension>", &fields.extension)
        .replace("<user>", &user)
        .replace("<year>", format!("{:04}", date.year()).as_str())
        .replace("<month>", format!("{:02}", date.month()).as_str())
        .replace("<day>", format!("{:02}", date.day()).as_str())
        .replace("<hour>", format!("{:02}", date.hour()).as_str())
        .replace("<minute>", format!("{:02}", date.minute()).as_str())
        .replace("<second>", format!("{:02}", date.second()).as_str())
        .replace("<camera_make>", &camera_make)
        .replace("<camera_model>", &camera_model)
        .replace("<album>", &album)
        .replace("<album_year>", format!("{:04}", album_year).as_str());
    pattern.insert_str(0, ".");
    pattern
}
