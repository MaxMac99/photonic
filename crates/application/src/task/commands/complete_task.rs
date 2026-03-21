use std::sync::Arc;

use derive_new::new;
use domain::{error::EntityNotFoundSnafu, task::TaskType, user::UserId};
use tracing::info;
use uuid::Uuid;

use crate::{
    error::{ApplicationError, ApplicationResult},
    task::ports::TaskRepository,
};

pub struct CompleteTaskCommand {
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub task_type: TaskType,
}

#[derive(new)]
pub struct CompleteTaskHandler {
    repository: Arc<dyn TaskRepository>,
}

impl CompleteTaskHandler {
    pub async fn handle(&self, command: CompleteTaskCommand) -> ApplicationResult<()> {
        let mut task = self
            .repository
            .find_by_reference_id(command.reference_id, command.task_type, command.user_id)
            .await?
            .ok_or_else(|| ApplicationError::Domain {
                source: EntityNotFoundSnafu {
                    entity: "TaskReference",
                    id: command.reference_id,
                }
                .build(),
            })?;

        let _event = task.complete()?;

        self.repository.save(&task).await?;

        info!(
            "Task {} completed (type={:?}, reference_id={:?}, user_id={:?}, duration={:?})",
            task.id,
            task.task_type,
            task.reference_id,
            task.user_id,
            task.duration()
        );

        Ok(())
    }
}
