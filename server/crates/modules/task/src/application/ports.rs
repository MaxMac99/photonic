use async_trait::async_trait;
use kernel::{error::DomainResult, UserId};
use uuid::Uuid;

use crate::domain::{Task, TaskType};

/// Write-side port (ADR 0002): task persistence plus the command-support
/// read the task lifecycle commands need. Shaped reads live on the query
/// ports (`application/queries`).
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_by_reference_id(
        &self,
        id: Uuid,
        task_type: TaskType,
        user_id: UserId,
    ) -> DomainResult<Option<Task>>;
    async fn save(&self, task: &Task) -> DomainResult<()>;
}
