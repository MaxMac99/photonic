use std::sync::Arc;

use derive_new::new;
use domain::{error::EntityNotFoundSnafu, task::TaskType, user::UserId};
use tracing::error;
use uuid::Uuid;

use crate::{
    error::{ApplicationError, ApplicationResult},
    task::ports::TaskRepository,
};

pub struct FailTaskCommand {
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub task_type: TaskType,
    pub error: String,
}

#[derive(new)]
pub struct FailTaskHandler {
    repository: Arc<dyn TaskRepository>,
}

impl FailTaskHandler {
    pub async fn handle(&self, command: FailTaskCommand) -> ApplicationResult<()> {
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

        task.fail(command.error.clone())?;

        self.repository.save(&task).await?;

        error!(
            "Task {} failed (type={:?}, reference_id={}, user_id={}): {}",
            task.id, task.task_type, task.reference_id, task.user_id, command.error
        );

        Ok(())
    }
}
