use std::sync::Arc;

use derive_new::new;
use tracing::debug;
use uuid::Uuid;

use crate::domain::task::{Task, TaskType};
use crate::{
    application::{error::ApplicationResult, task::ports::TaskRepository},
    domain::user::UserId,
};

pub struct StartTaskCommand {
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub task_type: TaskType,
}

#[derive(new)]
pub struct StartTaskHandler {
    repository: Arc<dyn TaskRepository>,
}

impl StartTaskHandler {
    pub async fn handle(&self, command: StartTaskCommand) -> ApplicationResult<()> {
        let mut task = self
            .repository
            .find_by_reference_id(command.reference_id, command.task_type, command.user_id)
            .await?
            .unwrap_or_else(|| Task::new(command.task_type, command.reference_id, command.user_id));

        task.start()?;

        self.repository.save(&task).await?;

        debug!(
            "Task {} started (type={:?}, reference_id={}, user_id={:?})",
            task.id, task.task_type, task.reference_id, task.user_id
        );

        Ok(())
    }
}
