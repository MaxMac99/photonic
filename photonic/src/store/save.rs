use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
use filenamify::filenamify;
use path_clean::PathClean;
use tokio::fs;

use crate::{
    error::{FileAlreadyExistsSnafu, NoFileExtensionSnafu, OutsideBaseStorageSnafu, Result},
    store::{path::PathOptions, Store},
};

impl Store {
    pub async fn import_file<P>(&self, options: &PathOptions, path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let destination = self.to_path(options);
        let destination = self.config.storage.base_path.join(destination);
        if destination.exists() {
            return Err(FileAlreadyExistsSnafu.build());
        }

        let destination = self.prepare_destination(&destination).await?;

        fs::rename(&path, &destination).await?;

        Ok(destination)
    }

    async fn prepare_destination(&self, destination: &PathBuf) -> Result<PathBuf> {
        let destination = destination.clean();
        if !destination.starts_with(&self.config.storage.base_path) {
            return OutsideBaseStorageSnafu.fail();
        }
        if destination.extension().is_none() {
            return NoFileExtensionSnafu.fail();
        }
        fs::create_dir_all(&destination.parent().unwrap()).await?;
        Ok(destination)
    }

    fn to_path(&self, options: &PathOptions) -> PathBuf {
        let mut result = self.config.storage.pattern.to_string();

        let username_filename = filenamify(&options.username);
        result = result.replace("<user>", &username_filename);

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
        result = result.replace("<camera_make>", &camera_make.replace(" ", "_"));

        let camera_model = options.camera_model.as_deref().unwrap_or("Unknown");
        result = result.replace("<camera_model>", &camera_model.replace(" ", "_"));

        result = result.replace("<filename>", &options.filename);
        result = result.replace("<extension>", &options.extension);

        if result.starts_with("/") {
            return PathBuf::from(&result[1..]);
        }

        PathBuf::from(result)
    }
}
