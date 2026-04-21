use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::{medium::events::MediumCreatedEvent, task::TaskType};
use tracing::{debug, info, instrument};

use crate::{
    error::ApplicationResult,
    event_bus::EventProcessor,
    task::commands::{CreateTaskCommand, CreateTaskHandler},
};

#[derive(new)]
pub struct TaskCreationListeners {
    create_task_handler: Arc<CreateTaskHandler>,
}

#[async_trait]
impl EventProcessor<MediumCreatedEvent> for TaskCreationListeners {
    type Error = crate::error::ApplicationError;

    #[instrument(name = "TaskCreationListeners::MediumCreatedEvent", skip(self, event))]
    async fn process(&self, event: &MediumCreatedEvent) -> ApplicationResult<()> {
        let item = &event.initial_item;
        info!(
            "Creating metadata extraction task for medium_id={} (leading_item_id={})",
            event.medium_id, item.id,
        );

        self.create_task_handler
            .handle(CreateTaskCommand {
                reference_id: item.id,
                user_id: event.user_id,
                task_type: TaskType::MetadataExtraction,
                file_location: item
                    .locations
                    .first()
                    .expect("Item must have a location")
                    .clone(),
            })
            .await?;

        debug!(
            "Created MetadataExtraction task for medium_id={}",
            event.medium_id
        );

        Ok(())
    }
}
