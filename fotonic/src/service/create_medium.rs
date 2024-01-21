use std::{
    backtrace::Backtrace,
    fmt::Debug,
    path::{Path, PathBuf},
};

use axum::{body::Bytes, BoxError};
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use futures::TryFutureExt;
use futures_util::{io, Stream, TryStreamExt};
use mime::Mime;
use mongodb::bson::oid::ObjectId;
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::{fs, fs::File, io::BufWriter, join};
use tokio_util::io::StreamReader;
use tracing::debug;

use meta::MetaError;

use crate::{
    entities::{Album, Medium, MediumItem, MediumType},
    repository::SaveMediumError,
    service::Service,
    store::{ImportError, PathOptions},
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

#[derive(Snafu, Debug)]
pub enum CreateMediumError {
    #[snafu(display(
        "Could not create file for stream at {path:?}: {source}"
    ))]
    StreamCreateFile {
        path: PathBuf,
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not stream to file: {source}"))]
    StreamWriteFile {
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not get metadata for file at {path:?}: {source}"))]
    FileMetadata {
        path: PathBuf,
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not get meta information"), context(false))]
    MetaError {
        #[snafu(backtrace)]
        source: MetaError,
    },
    #[snafu(display("Could not import file"), context(false))]
    Import {
        #[snafu(backtrace)]
        source: ImportError,
    },
    #[snafu(display("Could not save medium"), context(false))]
    SaveMedium {
        #[snafu(backtrace)]
        source: SaveMediumError,
    },
    #[snafu(display("Could not find the date when this medium was taken"))]
    NoDateTaken { backtrace: Backtrace },
    #[snafu(display("Could not get album"), context(false))]
    GetAlbum {
        source: mongodb::error::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not find album with id {album_id}"))]
    WrongAlbum {
        album_id: ObjectId,
        backtrace: Backtrace,
    },
}

impl Service {
    pub async fn store_stream_temporarily<S, E>(
        &self,
        extension: &str,
        stream: S,
    ) -> Result<PathBuf, CreateMediumError>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        let temp_path = self.store.get_temp_file_path(extension);
        let body_with_error =
            stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_error);
        futures::pin_mut!(body_reader);

        let file =
            File::create(&temp_path)
                .await
                .context(StreamCreateFileSnafu {
                    path: temp_path.clone(),
                })?;
        let mut buffer = BufWriter::new(file);

        tokio::io::copy(&mut body_reader, &mut buffer)
            .await
            .context(StreamWriteFileSnafu)?;

        debug!("Uploaded file temporarily to {}", temp_path.display());

        Ok(temp_path)
    }

    async fn test(&self) -> Result<(), CreateMediumError> {
        self.meta.read_file("Something stupid", false).await?;
        Ok(())
    }

    pub async fn create_medium<P>(
        &self,
        input: CreateMediumInput,
        path: P,
    ) -> Result<ObjectId, CreateMediumError>
    where
        P: AsRef<Path> + Debug,
    {
        let (file_size, path_opts) =
            self.create_path_options(&input, &path).await?;

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

        let id = self
            .repo
            .create_medium(medium)
            .or_else(|err| async {
                // Remove file if it could not store metadata
                let _ = fs::remove_file(&target_path).await;
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
    ) -> Result<(u64, PathOptions), CreateMediumError>
    where
        P: AsRef<Path> + Debug,
    {
        let (size, meta_info, album) = join!(
            fs::metadata(&path),
            self.meta.read_file(&path, true),
            self.get_album(input)
        );
        let size = size
            .context(FileMetadataSnafu {
                path: path.as_ref().to_path_buf(),
            })?
            .len();
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

    async fn get_album(
        &self,
        input: &CreateMediumInput,
    ) -> Result<Option<Album>, CreateMediumError> {
        let mut album: Option<Album> = None;
        if let Some(album_id) = input.album_id {
            album = self.repo.get_album_by_id(album_id).await?;
            if album.is_none() {
                return WrongAlbumSnafu { album_id }.fail();
            }
        }
        Ok(album)
    }
}
