use std::path::{Path, PathBuf};

use axum::body::Bytes;
use axum::BoxError;
use chrono::{Datelike, DateTime, Utc};
use futures::TryFutureExt;
use futures_util::{FutureExt, io, Stream, TryStreamExt};
use mongodb::bson::oid::ObjectId;
use tokio::{fs, join};
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

use crate::entities::{Album, Medium, MediumItem, MediumType};
use crate::errors::{Error, MediumError};
use crate::service::inputs::CreateMediumInput;
use crate::service::Service;
use crate::store::PathOptions;

impl Service {
    pub async fn store_stream_temporarily<S, E>(&self, extension: &str, stream: S) -> Result<PathBuf, Error>
        where
            S: Stream<Item=Result<Bytes, E>>,
            E: Into<BoxError>,
    {
        let temp_path = self.store.get_temp_file_path(extension);
        let body_with_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_error);
        futures::pin_mut!(body_reader);

        let mut file = BufWriter::new(File::create(&temp_path).await?);

        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok(temp_path)
    }

    pub async fn create_medium<P>(&self, input: CreateMediumInput, path: P) -> Result<ObjectId, Error>
        where P: AsRef<Path>
    {
        let (file_size, path_opts) = self.create_path_options(&input, &path).await?;

        let target_path = self.store.import_file(&path_opts, &path).await?;

        let medium = Medium {
            id: None,
            medium_type: MediumType::Photo,
            date_taken: path_opts.date,
            timezone: path_opts.timezone,
            originals: vec![MediumItem {
                id: None,
                mime: input.mime,
                filename: String::from(path_opts.filename),
                path: target_path.clone(),
                width: 0,
                height: 0,
                filesize: file_size,
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

        let id = self.repo.create_medium(medium)
            .or_else(|err| async {
                // Remove file if it could not store metadata
                fs::remove_file(&target_path).await?;
                Err(err)
            })
            .await?
            .inserted_id
            .as_object_id()
            .expect("Could not interpret inserted id as object id");
        Ok(id)
    }

    async fn create_path_options<P>(&self, input: &CreateMediumInput, path: P) -> Result<(u64, PathOptions), Error>
        where P: AsRef<Path>
    {
        let (size, meta_info, album) = join!(fs::metadata(&path), self.meta.read_file(&path), self.get_album(input));
        let size = size?.len();
        let meta_info = meta_info.map_err(|err| MediumError::UnsupportedFile)?;

        let date_taken = input.date_taken
            .or(meta_info.date)
            .ok_or(MediumError::NoDateTaken)?;
        let timezone = date_taken.timezone().local_minus_utc();
        let date_taken: DateTime<Utc> = date_taken.into();

        let album = album?;
        let name = album.as_ref().map(|album| album.name.clone());
        let year = album.as_ref().and_then(|album| album.first_date)
            .map(|date| date.year() as u32);
        let path_opts = PathOptions {
            album: name,
            album_year: year,
            date: date_taken,
            camera_make: meta_info.camera_make,
            camera_model: meta_info.camera_model,
            timezone,
            filename: input.filename.clone(),
            extension: input.extension.clone(),
        };
        Ok((size, path_opts))
    }

    async fn get_album(&self, input: &CreateMediumInput) -> Result<Option<Album>, Error> {
        let mut album: Option<Album> = None;
        if let Some(album_id) = input.album_id {
            album = self.repo.get_album_by_id(album_id).await
                .map_err(|err| Error::Internal(err.to_string()))?;
            if album.is_none() {
                return Err(MediumError::WrongAlbum.into());
            }
        }
        Ok(album)
    }
}
