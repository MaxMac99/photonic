use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use axum::{body::Bytes, BoxError};
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use futures::{TryFuture, TryFutureExt};
use futures_util::{io, Stream, TryStreamExt};
use mime::Mime;
use snafu::OptionExt;
use tokio::{fs, fs::File, io::BufWriter, join};
use tokio_util::io::StreamReader;
use tracing::{debug, error};
use uuid::Uuid;

use meta::MetaInfo;

use crate::{
    error::{Error, NoDateTakenSnafu, Result},
    model::{Album, FileItem, Medium, MediumItem, MediumItemType, MediumType, StoreLocation},
    service::{CreateUserInput, Service},
    store::PathOptions,
};

#[derive(Debug, Clone)]
pub struct CreateMediumInput {
    pub user: CreateUserInput,
    pub album_id: Option<Uuid>,
    pub filename: String,
    pub extension: String,
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub mime: Mime,
    pub priority: i32,
}

impl Service {
    pub async fn create_medium_from_stream<S, E>(
        &self,
        input: CreateMediumInput,
        stream: S,
    ) -> Result<Uuid>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        self.store_stream_temporarily(&input.extension.clone(), stream, |temp_path| async move {
            self.create_medium(input, &temp_path).await
        })
        .await
    }

    pub async fn create_medium<P>(&self, input: CreateMediumInput, path: P) -> Result<Uuid>
    where
        P: AsRef<Path> + Debug,
    {
        self.create_or_update_user(input.user.clone()).await?;

        let (file_size, meta_info, path_opts) = self
            .create_path_options(
                &path,
                input.user.username.unwrap(),
                input.album_id,
                input.date_taken,
                input.filename,
                input.extension,
            )
            .await?;

        self.store
            .import_file(&path_opts.clone(), &path, |target_path| async {
                let medium = Medium {
                    id: Uuid::new_v4(),
                    owner: input.user.id,
                    medium_type: MediumType::Photo,
                    originals: vec![Self::create_medium_item(
                        input.mime,
                        input.priority,
                        file_size,
                        meta_info,
                        path_opts,
                        target_path,
                    )],
                    album: None,
                    tags: input.tags.clone(),
                    previews: vec![],
                    edits: vec![],
                    sidecars: vec![],
                };

                self.repo.create_medium(medium).await
            })
            .await
    }
}
