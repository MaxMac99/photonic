use async_trait::async_trait;
use kernel::{error::DomainResult, UserId};

use crate::domain::{Task, TaskFilter};

/// Read-side port for task queries (ADR 0002).
#[async_trait]
pub trait TaskQueryPort: Send + Sync {
    async fn find_all(&self, filter: TaskFilter, user_id: UserId) -> DomainResult<Vec<Task>>;
}
