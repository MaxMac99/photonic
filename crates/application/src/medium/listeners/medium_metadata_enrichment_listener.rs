use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::{medium::camera::GpsCoordinates, metadata::events::MetadataExtractedEvent};
use tracing::{debug, info, instrument};

use crate::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    medium::commands::{EnrichMediumWithMetadataCommand, EnrichMediumWithMetadataHandler},
};

#[derive(new)]
pub struct MediumMetadataEnrichmentListener {
    handler: Arc<EnrichMediumWithMetadataHandler>,
}

#[async_trait]
impl EventProcessor<MetadataExtractedEvent> for MediumMetadataEnrichmentListener {
    #[instrument(
        name = "MediumMetadataEnrichmentListener::MetadataExtractedEvent",
        skip(self, event),
        fields(
            event = "MetadataExtractedEvent",
            medium_id = %event.medium_id,
            medium_item_id = %event.leading_item_id,
            user_id = %event.owner_id,
        )
    )]
    async fn process(&self, event: MetadataExtractedEvent) -> ApplicationResult<()> {
        info!(
            "Enriching medium metadata for medium_id={} (leading_item_id={})",
            event.medium_id, event.leading_item_id,
        );

        // Extract relevant fields from metadata (anti-corruption layer)
        let taken_at = event
            .metadata
            .camera_info
            .as_ref()
            .and_then(|c| c.capture_date)
            .or(event.metadata.file_info.file_modified_at);

        let camera_make = event
            .metadata
            .camera_info
            .as_ref()
            .and_then(|c| c.make.clone());

        let camera_model = event
            .metadata
            .camera_info
            .as_ref()
            .and_then(|c| c.model.clone());

        let gps_coordinates =
            event.metadata.location.as_ref().and_then(|loc| {
                GpsCoordinates::new(loc.latitude, loc.longitude, loc.altitude).ok()
            });

        self.handler
            .handle(EnrichMediumWithMetadataCommand {
                medium_id: event.medium_id,
                owner_id: event.owner_id,
                taken_at,
                camera_make,
                camera_model,
                gps_coordinates,
            })
            .await?;

        debug!("Enriched medium metadata for medium_id={}", event.medium_id);

        Ok(())
    }
}
