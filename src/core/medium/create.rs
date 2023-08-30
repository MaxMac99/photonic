use std::ffi::OsStr;
use std::path::Path;

use chrono::{Datelike, DateTime, FixedOffset, Utc};
use mime::Mime;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

use crate::common::meta::MetaInfo;
use crate::models::{Album, AppError, Medium, MediumItem, MediumType};
use crate::repository::{MongoRepo, PhotoRepo};
use crate::repository::photo::PathOptions;

#[derive(Deserialize)]
pub struct CreateMediumOpts {
    album_id: Option<ObjectId>,
    filename: String,
    #[serde(default)]
    tags: Vec<String>,
    date_taken: Option<DateTime<FixedOffset>>,
}

pub async fn create_medium(db: &MongoRepo, store: &PhotoRepo, opts: &CreateMediumOpts, mime: Mime, content: &[u8]) -> Result<ObjectId, AppError> {
    let metainfo = MetaInfo::from(content)?;

    if metainfo.mimetype != mime {
        return Err(AppError::WrongType {
            file: metainfo.mimetype,
            content_type: mime,
        });
    }

    let (filename, extension) = stem_and_extension_from_filename(&opts.filename)
        .ok_or(AppError::WrongFilename)?;
    let date_taken = opts.date_taken.or(metainfo.date)
        .ok_or(AppError::NoDateTaken)?;
    let mut album: Option<Album> = None;
    if let Some(album_id) = opts.album_id {
        album = db.get_album_by_id(album_id).await
            .map_err(|_| AppError::UnknownError)?;
        if album.is_none() {
            return Err(AppError::AlbumNotFound(album_id.to_string()));
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

    let result = store.save_file(&path_opts, content).await?;

    let medium = Medium {
        id: None,
        medium_type: MediumType::Photo,
        date_taken,
        originals: vec![MediumItem {
            id: None,
            mime,
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
        tags: opts.tags.clone(),
        preview: None,
        edits: vec![],
        sidecars: vec![],
    };

    let result = db.create_medium(medium)
        .await
        .map_err(|_| AppError::UnknownError)?;
    let id = result.inserted_id
        .as_object_id()
        .ok_or(AppError::UnknownError)?;
    Ok(id)
}

fn stem_and_extension_from_filename(filename: &str) -> Option<(&str, &str)> {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(OsStr::to_str)?;
    let extension = path.extension().and_then(OsStr::to_str)?;
    Some((stem, extension))
}
