use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::medium::events::MediumEvent;
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
impl EventProcessor<MediumEvent> for MoveToPermanentStorageListener {
    #[instrument(
        name = "MoveToPermanentStorageListener::MediumEvent",
        skip(self, event),
    )]
    async fn process(&self, event: MediumEvent) -> ApplicationResult<()> {
        let MediumEvent::MediumUpdated(event) = event else { return Ok(()) };

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
