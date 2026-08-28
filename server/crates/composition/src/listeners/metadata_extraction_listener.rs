use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use kernel::{app_error::ApplicationResult, event_bus::EventProcessor};
use medium::domain::events::MediumCreatedEvent;
use metadata::application::commands::{ExtractMetadataCommand, ExtractMetadataHandler};
use tracing::{info, instrument};

#[derive(new)]
pub struct MetadataExtractionListeners {
    extract_metadata_handler: Arc<ExtractMetadataHandler>,
}

#[async_trait]
impl EventProcessor<MediumCreatedEvent> for MetadataExtractionListeners {
    type Error = kernel::app_error::ApplicationError;

    #[instrument(
        name = "MetadataExtractionListener::MediumCreatedEvent",
        skip(self, event)
    )]
    async fn process(&self, event: &MediumCreatedEvent) -> ApplicationResult<()> {
        let item = &event.initial_item;
        info!(
            "Starting metadata extraction task for medium_id={} (leading_item_id={}, user_id={})",
            event.medium_id, item.id, event.user_id
        );

        self.extract_metadata_handler
            .handle(ExtractMetadataCommand {
                medium_id: event.medium_id,
                leading_item_id: item.id,
                user_id: event.user_id,
                file_location: item
                    .locations
                    .first()
                    .expect("Item must have a location")
                    .clone(),
            })
            .await?;

        Ok(())
    }
}
