use async_trait::async_trait;
use kernel::{error::DomainResult, UserId};
use uuid::Uuid;

use crate::domain::{Task, TaskFilter, TaskType};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_by_reference_id(
        &self,
        id: Uuid,
        task_type: TaskType,
        user_id: UserId,
    ) -> DomainResult<Option<Task>>;
    async fn find_all(&self, filter: TaskFilter, user_id: UserId) -> DomainResult<Vec<Task>>;
    async fn save(&self, task: &Task) -> DomainResult<()>;
}
