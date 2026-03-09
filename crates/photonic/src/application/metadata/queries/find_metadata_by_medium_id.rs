use std::sync::Arc;

use derive_new::new;
use snafu::OptionExt;
use tracing::{debug, error, info, instrument};

use crate::{
    application::{error::ApplicationResult, metadata::ports::MetadataRepository},
    domain::{
        error::EntityNotFoundSnafu,
        medium::MediumId,
        metadata::Metadata,
        user::UserId,
    },
};

#[derive(Debug)]
pub struct FindMetadataByMediumIdQuery {
    pub medium_id: MediumId,
    pub user_id: UserId,
}

#[derive(new)]
pub struct FindMetadataByMediumIdHandler {
    metadata_repository: Arc<dyn MetadataRepository>,
}

impl FindMetadataByMediumIdHandler {
    #[instrument(skip(self), fields(
        medium_id = %query.medium_id,
        user_id = %query.user_id,
    ))]
    pub async fn handle(&self, query: FindMetadataByMediumIdQuery) -> ApplicationResult<Metadata> {
        info!("Finding metadata by medium ID");

        let metadata = self
            .metadata_repository
            .find_by_medium_id(query.medium_id)
            .await
            .and_then(|m| {
                m.context(EntityNotFoundSnafu {
                    entity: "Metadata",
                    id: query.medium_id,
                })
            })
            .map_err(|e| {
                error!(error = ?e, "Failed to find metadata");
                e
            })?;

        debug!("Metadata retrieved successfully");

        Ok(metadata)
    }
}
