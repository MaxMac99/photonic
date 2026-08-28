use std::sync::Arc;

use derive_new::new;
use kernel::{ApplicationResult, FileLocation, UserId};
use tracing::debug;
use uuid::Uuid;

use crate::{
    application::ports::TaskRepository,
    domain::{Task, TaskType},
};

pub struct CreateTaskCommand {
    pub reference_id: Uuid,
    pub user_id: UserId,
    pub task_type: TaskType,
    pub file_location: FileLocation,
}

#[derive(new)]
pub struct CreateTaskHandler {
    repository: Arc<dyn TaskRepository>,
}

impl CreateTaskHandler {
    pub async fn handle(&self, command: CreateTaskCommand) -> ApplicationResult<Task> {
        let (task, _event) = Task::new(command.task_type, command.reference_id, command.user_id);

        self.repository.save(&task).await?;

        debug!(
            "Created task: id={}, type={:?}, reference_id={}, user_id={:?}",
            task.id, task.task_type, task.reference_id, task.user_id
        );

        Ok(task)
    }
}
