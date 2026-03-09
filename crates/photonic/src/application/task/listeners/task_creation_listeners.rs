use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use tracing::{debug, info, instrument};

use crate::domain::task::TaskType;
use crate::{
    application::{
        error::ApplicationResult,
        event_bus::EventProcessor,
        task::commands::{CreateTaskCommand, CreateTaskHandler},
    },
    domain::medium::events::MediumCreatedEvent,
};

#[derive(new)]
pub struct TaskCreationListeners {
    create_task_handler: Arc<CreateTaskHandler>,
}

#[async_trait]
impl EventProcessor<MediumCreatedEvent> for TaskCreationListeners {
    #[instrument(
        name = "TaskCreationListeners::MediumCreatedEvent",
        skip(self, event),
        fields(
            event = "MediumCreatedEvent",
            medium_id = %event.medium_id,
            medium_item_id = %event.leading_item_id,
            user_id = %event.user_id,
        )
    )]
    async fn process(&self, event: MediumCreatedEvent) -> ApplicationResult<()> {
        info!(
            "Creating metadata extraction task for medium_id={} (leading_item_id={})",
            event.medium_id, event.leading_item_id,
        );

        self.create_task_handler
            .handle(CreateTaskCommand {
                reference_id: event.leading_item_id,
                user_id: event.user_id,
                task_type: TaskType::MetadataExtraction,
                file_location: event.leading_item_location.clone(),
            })
            .await?;

        debug!(
            "Created MetadataExtraction task for medium_id={}",
            event.medium_id
        );

        Ok(())
    }
}
