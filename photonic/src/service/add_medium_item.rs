use crate::{
    error::Result,
    model::{FileItem, MediumItem, MediumItemType, StoreLocation},
    service::CreateUserInput,
    store::PathOptions,
    Service,
};
use axum::{body::Bytes, BoxError};
use chrono::{DateTime, FixedOffset, Utc};
use futures::Stream;
use futures_util::TryFutureExt;
use meta::MetaInfo;
use mime::Mime;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AddMediumItemInput {
    pub user: CreateUserInput,
    pub item_type: AddMediumItemType,
    pub medium_id: Uuid,
    pub filename: String,
    pub extension: String,
    pub date_taken: Option<DateTime<FixedOffset>>,
    pub mime: Mime,
    pub priority: i32,
}

#[derive(Debug, Copy, Clone)]
pub enum AddMediumItemType {
    Original,
    Edit,
    Preview,
    Sidecar,
}

impl Service {
    pub async fn add_raw_file_from_stream<S, E>(
        &self,
        input: AddMediumItemInput,
        stream: S,
    ) -> Result<Uuid>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        self.store_stream_temporarily(&input.extension.clone(), stream, |temp_path| async move {
            self.add_raw_file(input, &temp_path).await
        })
        .await
    }

    pub async fn add_raw_file<P>(&self, input: AddMediumItemInput, path: P) -> Result<Uuid>
    where
        P: AsRef<Path> + Debug,
    {
        self.create_or_update_user(input.user.clone()).await?;

        let medium = self.repo.get_medium(input.medium_id, input.user.id).await?;
        let (file_size, meta_info, path_opts) = self
            .create_path_options(
                &path,
                input.user.username.unwrap(),
                medium.album,
                input.date_taken,
                input.filename,
                input.extension,
            )
            .await?;

        self.store
            .import_file(&path_opts.clone(), &path, |target_path| async move {
                let medium_item = Self::create_medium_item(
                    input.mime,
                    input.priority,
                    file_size,
                    meta_info,
                    path_opts,
                    target_path.clone(),
                );
                if let AddMediumItemType::Sidecar = input.item_type {
                    self.repo
                        .add_sidecar(input.medium_id, medium_item.file)
                        .await
                } else {
                    let medium_item_type = match input.item_type {
                        AddMediumItemType::Original => MediumItemType::Original,
                        AddMediumItemType::Edit => MediumItemType::Edit,
                        AddMediumItemType::Preview => MediumItemType::Preview,
                        AddMediumItemType::Sidecar => MediumItemType::Original, // Not possible
                    };
                    self.repo
                        .add_medium_item(input.medium_id, medium_item_type, medium_item)
                        .await
                }
            })
            .await
    }

    pub(crate) fn create_medium_item(
        mime: Mime,
        priority: i32,
        file_size: u64,
        meta_info: MetaInfo,
        path_opts: PathOptions,
        target_path: PathBuf,
    ) -> MediumItem {
        MediumItem {
            file: FileItem {
                id: Uuid::new_v4(),
                mime,
                filename: String::from(path_opts.filename),
                path: target_path,
                filesize: file_size,
                last_saved: Utc::now().naive_utc(),
                location: StoreLocation::Originals,
                priority,
            },
            width: meta_info.width as u32,
            height: meta_info.height as u32,
            date_taken: path_opts
                .date
                .with_timezone(&FixedOffset::east_opt(path_opts.timezone).unwrap()),
        }
    }
}
