use std::sync::Arc;

use chrono::{DateTime, FixedOffset};
use derive_new::new;
use domain::{
    error::EntityNotFoundSnafu,
    medium::{camera::GpsCoordinates, MediumId},
    user::UserId,
};
use snafu::OptionExt;
use tracing::{debug, warn};

use crate::{
    error::ApplicationResult,
    medium::ports::{MediumRepository, PublishMediumEvent},
};

pub struct EnrichMediumWithMetadataCommand {
    pub medium_id: MediumId,
    pub owner_id: UserId,
    pub taken_at: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_coordinates: Option<GpsCoordinates>,
}

#[derive(new)]
pub struct EnrichMediumWithMetadataHandler {
    medium_repository: Arc<dyn MediumRepository>,
    event_bus: Arc<dyn PublishMediumEvent>,
}

impl EnrichMediumWithMetadataHandler {
    pub async fn handle(&self, command: EnrichMediumWithMetadataCommand) -> ApplicationResult<()> {
        let mut medium = self
            .medium_repository
            .find_by_id(command.medium_id, command.owner_id)
            .await?
            .context(EntityNotFoundSnafu {
                entity: "Medium",
                id: command.medium_id,
            })?;

        let event = medium.update_basic_metadata(
            command.taken_at,
            command.camera_make,
            command.camera_model,
            command.gps_coordinates,
        );

        self.medium_repository.save(&medium).await?;

        debug!(
            medium_id = %command.medium_id,
            taken_at = ?command.taken_at,
            has_gps = command.gps_coordinates.is_some(),
            "Medium enriched with metadata"
        );

        if let Err(e) = self.event_bus.publish(event).await {
            warn!(
                medium_id = %command.medium_id,
                error = %e,
                "Failed to publish MediumUpdatedEvent"
            );
        }

        Ok(())
    }
}
