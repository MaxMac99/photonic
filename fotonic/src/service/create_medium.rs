use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use axum::{body::Bytes, BoxError};
use bson::Uuid;
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use futures::TryFutureExt;
use futures_util::{io, Stream, TryStreamExt};
use mime::Mime;
use mongodb::bson::oid::ObjectId;
use snafu::OptionExt;
use tokio::{fs, fs::File, io::BufWriter, join};
use tokio_util::io::StreamReader;
use tracing::{debug, error};

use meta::MetaInfo;

use crate::{
    error::{NoDateTakenSnafu, Result},
    model::{Access, Album, FileItem, Medium, MediumItem, MediumType, StoreLocation},
    service::Service,
    store::PathOptions,
};

#[derive(Debug, Clone)]
pub struct CreateMediumInput {
    pub album_id: Option<ObjectId>,
    pub filename: String,
    pub extension: String,
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub mime: Mime,
}

impl Service {
    pub async fn store_stream_temporarily<S, E>(
        &self,
        extension: &str,
        stream: S,
    ) -> Result<PathBuf>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        let temp_path = self.store.get_temp_file_path(extension);
        let body_with_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_error);
        futures::pin_mut!(body_reader);

        let file = File::create(&temp_path).await?;
        let mut buffer = BufWriter::new(file);

        tokio::io::copy(&mut body_reader, &mut buffer).await?;

        debug!("Uploaded file temporarily to {}", temp_path.display());

        Ok(temp_path)
    }

    pub async fn create_medium<P>(
        &self,
        user_id: Uuid,
        username: String,
        input: CreateMediumInput,
        path: P,
    ) -> Result<ObjectId>
    where
        P: AsRef<Path> + Debug,
    {
        let (file_size, meta_info, path_opts) =
            self.create_path_options(&input, &path, username).await?;

        let target_path = self.store.import_file(&path_opts, &path).await?;
        let medium = Medium {
            id: None,
            access: Access { owner: user_id },
            medium_type: MediumType::Photo,
            date_taken: path_opts.date,
            timezone: path_opts.timezone,
            originals: vec![MediumItem {
                file: FileItem {
                    id: ObjectId::new(),
                    mime: input.mime,
                    filename: String::from(path_opts.filename),
                    path: target_path.clone(),
                    filesize: file_size,
                    last_saved: Utc::now(),
                    location: StoreLocation::Originals,
                },
                width: meta_info.width,
                height: meta_info.height,
                priority: 10,
            }],
            album: None,
            tags: input.tags.clone(),
            preview: None,
            edits: vec![],
            sidecars: vec![],
            additional_data: meta_info.additional_data,
        };

        let id = self
            .repo
            .create_medium(medium)
            .or_else(|err| async {
                // Remove file if it could not store metadata
                let full_path = self
                    .store
                    .get_full_path_from_relative(StoreLocation::Originals, &target_path);
                if let Err(remove_err) = fs::remove_file(&full_path).await {
                    error!("Could not delete file for rollback: {}", remove_err);
                }
                Err(err)
            })
            .await?
            .inserted_id
            .as_object_id()
            .expect("Could not interpret inserted id as object id");
        Ok(id)
    }

    async fn create_path_options<P>(
        &self,
        input: &CreateMediumInput,
        path: P,
        username: String,
    ) -> Result<(u64, MetaInfo, PathOptions)>
    where
        P: AsRef<Path> + Debug,
    {
        let (size, meta_info, album) = join!(
            fs::metadata(&path),
            self.meta.read_file(&path, true),
            self.get_album(input)
        );
        let size = size?.len();
        let meta_info = meta_info?;

        let date_taken = input
            .date_taken
            .or(meta_info.date)
            .context(NoDateTakenSnafu)?;
        let timezone = date_taken.timezone().local_minus_utc();
        let date_taken: DateTime<Utc> = date_taken.into();

        let album = album?;
        let name = album.as_ref().map(|album| album.name.clone());
        let year = album
            .as_ref()
            .and_then(|album| album.first_date)
            .map(|date| date.year() as u32);
        let path_opts = PathOptions {
            username,
            album: name,
            album_year: year,
            date: date_taken,
            camera_make: meta_info.camera_make.clone(),
            camera_model: meta_info.camera_model.clone(),
            timezone,
            filename: input.filename.clone(),
            extension: input.extension.clone(),
        };
        Ok((size, meta_info, path_opts))
    }

    async fn get_album(&self, input: &CreateMediumInput) -> Result<Option<Album>> {
        let mut album: Option<Album> = None;
        if let Some(album_id) = input.album_id {
            album = Some(self.repo.get_album_by_id(album_id).await?);
        }
        Ok(album)
    }
}
