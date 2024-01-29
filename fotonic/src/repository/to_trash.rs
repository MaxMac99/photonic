use std::backtrace::Backtrace;

use bson::{doc, oid::ObjectId};
use futures_util::FutureExt;
use mongodb::{error::ErrorKind, ClientSession};

use crate::{
    error::{Error, FindMediumByIdSnafu, Result},
    model::TrashItem,
    repository::Repository,
};

impl Repository {
    pub async fn move_to_trash(&self, id: ObjectId) -> Result<()> {
        let mut session = self.client.start_session(None).await?;
        session
            .with_transaction(
                (id, &self),
                |session, (id, repo)| {
                    repo.move_to_trash_internal(*id, session).boxed()
                },
                None,
            )
            .await
            .map_err(|err| -> Error {
                if let ErrorKind::Custom(_) = err.kind.as_ref() {
                    return FindMediumByIdSnafu { id }.build();
                }
                return Error::MongoDb {
                    source: err,
                    backtrace: Backtrace::force_capture(),
                };
            })?;
        Ok(())
    }

    async fn move_to_trash_internal(
        &self,
        id: ObjectId,
        session: &mut ClientSession,
    ) -> std::result::Result<(), mongodb::error::Error> {
        let medium = self
            .medium_col
            .find_one_with_session(doc! { "_id": id }, None, session)
            .await?
            .ok_or(mongodb::error::Error::custom(Error::FindMediumById {
                id,
                backtrace: Backtrace::force_capture(),
            }))?;

        let trash_item = TrashItem {
            id: ObjectId::new(),
            medium_id: id,
            deleted: chrono::offset::Utc::now(),
            medium: Some(medium),
            original: None,
            edit: None,
            sidecar: None,
            preview: None,
        };
        self.trash_col
            .insert_one_with_session(trash_item, None, session)
            .await?;
        self.medium_col
            .delete_one_with_session(doc! { "_id": id }, None, session)
            .await?;
        Ok(())
    }
}
