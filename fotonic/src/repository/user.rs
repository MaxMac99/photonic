use bson::{doc, Uuid};
use mongodb::{options::FindOneAndUpdateOptions, ClientSession};
use tracing::log::error;

use crate::{
    error::{FindUserByIdSnafu, NoQuotaLeftSnafu, Result},
    repository::Repository,
    service::CreateUserInput,
};

impl Repository {
    pub async fn create_or_update_user(&self, user: CreateUserInput) -> Result<()> {
        self.user_col
            .find_one_and_update(
                doc! {
                    "_id": user.id,
                },
                doc! {
                    "$set": {
                        "_id": user.id,
                        "username": user.username,
                        "email": user.email,
                        "givenName": user.given_name,
                        "quota": user.quota as i64,
                    },
                    "$setOnInsert": {
                        "quotaUsed": 0i64
                    }
                },
                FindOneAndUpdateOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }

    pub(crate) async fn update_user_used_quota(
        &self,
        id: Uuid,
        new_quota: u64,
        session: &mut ClientSession,
    ) -> mongodb::error::Result<()> {
        let update_result = self
            .user_col
            .update_one_with_session(
                doc! {
                    "_id": id,
                },
                vec![doc! {
                    "$set": {
                        "quotaUsed": {
                            "$sum": [
                                "$quotaUsed",
                                {
                                    "$cond": {
                                        "if": {
                                            "$lte": [
                                                {
                                                    "$add": [
                                                        "$quotaUsed", new_quota as i64
                                                    ]
                                                },
                                                "$quota"
                                            ]
                                        },
                                        "then": new_quota as i64,
                                        "else": 0i64
                                    }
                                }
                            ]
                        },
                    }
                }],
                None,
                session,
            )
            .await;
        if let Some(err) = update_result.as_ref().err() {
            error!("Updating error: {:?}", err);
        }
        let update_result = update_result?;
        if update_result.matched_count != 1 {
            return Err(mongodb::error::Error::custom(
                FindUserByIdSnafu { id }.build(),
            ));
        }
        if update_result.modified_count != 1 {
            return Err(mongodb::error::Error::custom(NoQuotaLeftSnafu.build()));
        }
        Ok(())
    }
}
