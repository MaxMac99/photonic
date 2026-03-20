use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::medium::events::MediumCreatedEvent;
use tracing::{info, instrument};

use crate::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    metadata::commands::{ExtractMetadataCommand, ExtractMetadataHandler},
};

#[derive(new)]
pub struct MetadataExtractionListeners {
    extract_metadata_handler: Arc<ExtractMetadataHandler>,
}

#[async_trait]
impl EventProcessor<MediumCreatedEvent> for MetadataExtractionListeners {
    #[instrument(
        name = "MetadataExtractionListenerMediumCreatedEvent",
        skip(self, event),
        fields(
            medium_id = %event.medium_id,
            medium_item_id = %event.leading_item_id,
            user_id = %event.user_id,
        )
    )]
    async fn process(&self, event: MediumCreatedEvent) -> ApplicationResult<()> {
        info!(
            "Starting metadata extraction task for medium_id={} (leading_item_id={}, user_id={})",
            event.medium_id, event.leading_item_id, event.user_id
        );

        self.extract_metadata_handler
            .handle(ExtractMetadataCommand {
                medium_id: event.medium_id,
                leading_item_id: event.leading_item_id,
                user_id: event.user_id,
                file_location: event.leading_item_location,
            })
            .await?;

        Ok(())
    }
}
