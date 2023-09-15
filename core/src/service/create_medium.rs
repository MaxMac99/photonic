use std::ffi::OsStr;
use std::path::Path;

use chrono::{Datelike, Utc};
use mongodb::bson::oid::ObjectId;

use meta::MetaInfo;

use crate::entities::{Album, Medium, MediumItem, MediumType};
use crate::errors::{Error, MediumError};
use crate::service::inputs::CreateMediumInput;
use crate::service::Service;
use crate::store::PathOptions;

impl Service {
    pub async fn create_medium(&self, input: CreateMediumInput, content: &[u8]) -> Result<ObjectId, Error> {
        let metainfo = MetaInfo::from(content)
            .map_err(|_| MediumError::UnsupportedFile)?;

        if metainfo.mimetype != input.mime {
            return Err(MediumError::MimeMismatch {
                given_mime: input.mime.to_string(),
                found_mime: metainfo.mimetype.to_string(),
            }.into());
        }

        let (filename, extension) = stem_and_extension_from_filename(&input.filename)
            .ok_or(MediumError::WrongFilename)?;
        let date_taken = input.date_taken
            .or(metainfo.date)
            .ok_or(MediumError::NoDateTaken)?;
        let mut album: Option<Album> = None;
        if let Some(album_id) = input.album_id {
            album = self.repo.get_album_by_id(album_id).await
                .map_err(|err| Error::Internal(err.to_string()))?;
            if album.is_none() {
                return Err(MediumError::WrongAlbum.into());
            }
        }

        let name = album.as_ref().map(|album| album.name.clone());
        let year = album.as_ref().and_then(|album| album.first_date)
            .map(|date| date.year() as u32);
        let path_opts = PathOptions {
            album: name,
            album_year: year,
            date: date_taken,
            camera_make: metainfo.camera_make,
            camera_model: metainfo.camera_model,
            filename: String::from(filename),
            extension: String::from(extension),
        };

        let result = self.store.save_file(&path_opts, content).await?;

        let medium = Medium {
            id: None,
            medium_type: MediumType::Photo,
            date_taken,
            originals: vec![MediumItem {
                id: None,
                mime: input.mime,
                filename: String::from(filename),
                path: result,
                width: 0,
                height: 0,
                filesize: content.len() as u64,
                last_saved: Utc::now(),
                original_store: true,
                priority: 10,
            }],
            album: None,
            tags: input.tags.clone(),
            preview: None,
            edits: vec![],
            sidecars: vec![],
        };

        let result = self.repo.create_medium(medium).await?;
        let id = result.inserted_id.as_object_id()
            .ok_or(MediumError::UnknownError(String::from("No inserted id")))?;
        Ok(id)
    }
}

fn stem_and_extension_from_filename(filename: &str) -> Option<(&str, &str)> {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(OsStr::to_str)?;
    let extension = path.extension().and_then(OsStr::to_str)?;
    Some((stem, extension))
}