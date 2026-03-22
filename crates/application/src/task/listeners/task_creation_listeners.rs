use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use domain::{medium::events::MediumEvent, task::TaskType};
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
impl EventProcessor<MediumEvent> for TaskCreationListeners {
    #[instrument(
        name = "TaskCreationListeners::MediumEvent",
        skip(self, event),
    )]
    async fn process(&self, event: MediumEvent) -> ApplicationResult<()> {
        let MediumEvent::MediumCreated(event) = event else { return Ok(()) };
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
