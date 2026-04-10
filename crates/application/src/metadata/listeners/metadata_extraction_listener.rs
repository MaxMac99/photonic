use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::medium::events::MediumEvent;
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
impl EventProcessor<MediumEvent> for MetadataExtractionListeners {
    #[instrument(
        name = "MetadataExtractionListener::MediumEvent",
        skip(self, event),
    )]
    async fn process(&self, event: &MediumEvent) -> ApplicationResult<()> {
        let MediumEvent::MediumCreated(event) = event else { return Ok(()) };
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
                file_location: item.locations.first().expect("Item must have a location").clone(),
            })
            .await?;

        Ok(())
    }
}
