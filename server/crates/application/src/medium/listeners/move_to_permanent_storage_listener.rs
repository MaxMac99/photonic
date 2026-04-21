use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::medium::events::MediumUpdatedEvent;
use tracing::{info, instrument};

use crate::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    medium::commands::{MoveToPermanentStorageCommand, MoveToPermanentStorageHandler},
};

#[derive(new)]
pub struct MoveToPermanentStorageListener {
    handler: Arc<MoveToPermanentStorageHandler>,
}

#[async_trait]
impl EventProcessor<MediumUpdatedEvent> for MoveToPermanentStorageListener {
    type Error = crate::error::ApplicationError;

    #[instrument(
        name = "MoveToPermanentStorageListener::MediumUpdatedEvent",
        skip(self, event)
    )]
    async fn process(&self, event: &MediumUpdatedEvent) -> ApplicationResult<()> {
        info!(
            medium_id = %event.medium_id,
            user_id = %event.owner_id,
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
