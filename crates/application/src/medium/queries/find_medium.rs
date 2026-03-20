use std::sync::Arc;

use derive_new::new;
use domain::{
    error::EntityNotFoundSnafu,
    medium::{Medium, MediumId},
    user::UserId,
};
use snafu::OptionExt;
use tracing::{debug, error, info, instrument};

use crate::{error::ApplicationResult, medium::ports::MediumRepository};

#[derive(Debug)]
pub struct FindMediumQuery {
    pub user_id: UserId,
    pub medium_id: MediumId,
}

#[derive(new)]
pub struct FindMediumHandler {
    medium_repository: Arc<dyn MediumRepository>,
}

impl FindMediumHandler {
    #[instrument(skip(self), fields(
        user_id = %query.user_id,
        medium_id = %query.medium_id,
    ))]
    pub async fn handle(&self, query: FindMediumQuery) -> ApplicationResult<Medium> {
        info!("Finding medium by ID");

        let medium = self
            .medium_repository
            .find_by_id(query.medium_id, query.user_id)
            .await
            .and_then(|m| {
                m.context(EntityNotFoundSnafu {
                    entity: "Medium",
                    id: query.medium_id,
                })
            })
            .map_err(|e| {
                error!(error = ?e, "Failed to find medium");
                e
            })?;

        debug!("Medium retrieved successfully");

        Ok(medium)
    }
}
