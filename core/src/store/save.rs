use std::path::PathBuf;

use chrono::{Datelike, Timelike};
use path_clean::PathClean;
use tokio::{fs, io::AsyncWriteExt};

use crate::errors::MediumError;
use crate::store::path_opts::PathOptions;
use crate::store::Store;

impl Store {
    pub async fn save_file(&self, options: &PathOptions, data: &[u8]) -> Result<PathBuf, MediumError> {
        let path = self.to_path(options);
        let destination = self.config.storage.base_path.join(&path).clean();
        if !destination.starts_with(&self.config.storage.base_path) {
            return Err(MediumError::WrongFilename);
        }

        if destination.extension().is_none() {
            return Err(MediumError::WrongFilename);
        }
        fs::create_dir_all(&destination.parent().unwrap()).await?;

        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(true)
            .open(&destination)
            .await?;

        file.write_all(data).await?;
        Ok(path)
    }

    fn to_path(&self, options: &PathOptions) -> PathBuf {
        let mut result = self.config.storage.pattern.to_string();

        let album = options.album.as_deref().unwrap_or("Unknown");
        result = result.replace("<album>", &album);

        let album_year = options.album_year.unwrap_or(options.date.year() as u32);
        let album_year = format!("{:04}", album_year);
        result = result.replace("<album_year>", &album_year);

        let year = format!("{:04}", options.date.year());
        result = result.replace("<year>", &year);

        let month = format!("{:02}", options.date.month());
        result = result.replace("<month>", &month);

        let day = format!("{:02}", options.date.day());
        result = result.replace("<day>", &day);

        let hour = format!("{:02}", options.date.hour());
        result = result.replace("<hour>", &hour);

        let minute = format!("{:02}", options.date.minute());
        result = result.replace("<minute>", &minute);

        let second = format!("{:02}", options.date.second());
        result = result.replace("<second>", &second);

        let camera_make = options.camera_make.as_deref().unwrap_or("Unknown");
        result =
            result.replace("<camera_make>", &camera_make.replace(" ", "_"));

        let camera_model = options.camera_model.as_deref().unwrap_or("Unknown");
        result =
            result.replace("<camera_model>", &camera_model.replace(" ", "_"));

        result = result.replace("<filename>", &options.filename);
        result = result.replace("<extension>", &options.extension);

        if result.starts_with("/") {
            return PathBuf::from(&result[1..]);
        }

        PathBuf::from(result)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::sync::Arc;

    use chrono::{TimeZone, Utc};

    use crate::config::{Config, Storage};
    use crate::store::Store;

    use super::PathOptions;

    #[test]
    fn path_from_options() {
        let config = Config {
            storage: Storage {
                base_path: PathBuf::new(),
                pattern: String::from("/<album_year>/<album>/<year><month><day>/<camera_make>_<camera_model>/<filename>_<hour><minute><second>.<extension>"),
            }
        };
        let store = Store {
            config: Arc::new(config),
        };
        let opts = PathOptions {
            album: Some(String::from("Album with space")),
            album_year: Some(2022),
            date: Utc.with_ymd_and_hms(2023, 2, 1, 8, 7, 6).unwrap().fixed_offset(),
            camera_make: Some(String::from("Sony Alpha")),
            camera_model: Some(String::from("A7S III")),
            filename: String::from("DSC 123"),
            extension: String::from("jpg"),
        };

        assert_eq!(store.to_path(&opts), PathBuf::from("/2022/Album with space/20230201/Sony_Alpha_A7S_III/DSC 123_080706.jpg"))
    }

    #[test]
    fn path_from_minimal_options() {
        let config = Config {
            storage: Storage {
                base_path: PathBuf::new(),
                pattern: String::from("/<album_year>/<album>/<year><month><day>/<camera_make>_<camera_model>/<filename>_<hour><minute><second>.<extension>"),
            }
        };
        let store = Store {
            config: Arc::new(config),
        };
        let opts = PathOptions::default()
            .filename(String::from("DSC 123"))
            .extension(String::from("jpg"));

        assert_eq!(
            store.to_path(&opts),
            PathBuf::from(
                "/Unknown/Unknown/UnknownUnknownUnknown/Unknown_Unknown/DSC 123_UnknownUnknownUnknown.jpg"
            )
        )
    }
}
