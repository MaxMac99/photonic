use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use tracing::{info, instrument};

use crate::application::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    medium::commands::{MoveToPermanentStorageCommand, MoveToPermanentStorageHandler},
};
use crate::domain::medium::events::MediumUpdatedEvent;

#[derive(new)]
pub struct MoveToPermanentStorageListener {
    handler: Arc<MoveToPermanentStorageHandler>,
}

#[async_trait]
impl EventProcessor<MediumUpdatedEvent> for MoveToPermanentStorageListener {
    #[instrument(
        name = "MoveToPermanentStorageListener::MediumUpdatedEvent",
        skip(self, event),
        fields(
            event = "MediumUpdatedEvent",
            medium_id = %event.medium_id,
            user_id = %event.owner_id,
        )
    )]
    async fn process(&self, event: MediumUpdatedEvent) -> ApplicationResult<()> {
        info!(
            "Moving medium items to permanent storage for medium_id={}",
            event.medium_id,
        );

        self.handler
            .handle(MoveToPermanentStorageCommand {
                medium_id: event.medium_id,
                user_id: event.owner_id,
            })
            .await
    }
}