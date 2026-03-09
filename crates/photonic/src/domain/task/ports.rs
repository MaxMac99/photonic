use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    error::DomainResult,
    task::{Task, TaskFilter, TaskId, TaskStatus},
    user::UserId,
};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_by_id(&self, id: TaskId) -> DomainResult<Option<Task>>;
    async fn find_by_reference(
        &self,
        task_type: &str,
        reference_id: Uuid,
    ) -> DomainResult<Option<Task>>;
    async fn find_all(&self, filter: TaskFilter, user_id: UserId) -> DomainResult<Vec<Task>>;
    async fn find_pending_by_type(&self, task_type: &str) -> DomainResult<Vec<Task>>;
    async fn save(&self, task: &Task) -> DomainResult<()>;
    async fn update_status(&self, id: TaskId, status: TaskStatus) -> DomainResult<()>;
}
