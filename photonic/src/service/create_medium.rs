use std::{fmt::Debug, path::Path};

use axum::{body::Bytes, BoxError};
use chrono::{DateTime, FixedOffset};
use futures::TryFuture;
use futures_util::Stream;
use mime::Mime;
use uuid::Uuid;

use crate::{error::Result, service::CreateUserInput};

impl Service {
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
