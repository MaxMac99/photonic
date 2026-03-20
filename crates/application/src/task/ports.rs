use async_trait::async_trait;
use domain::{
    error::DomainResult,
    task::{Task, TaskFilter, TaskType},
    user::UserId,
};
use uuid::Uuid;

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
