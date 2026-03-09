use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::task::TaskType;
use crate::{
    application::task::TaskRepository,
    domain::{
        error::DomainResult,
        task::{Task, TaskFilter},
        user::UserId,
    },
};

mod entity;
mod find_all;
mod find_by_reference_id;
mod save;
mod task_types;

pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl PostgresTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for PostgresTaskRepository {
    #[tracing::instrument(skip(self), fields(task_id = %id, user_id = %user_id))]
    async fn find_by_reference_id(
        &self,
        id: Uuid,
        task_type: TaskType,
        user_id: UserId,
    ) -> DomainResult<Option<Task>> {
        self.find_by_reference_id_impl(id, task_type, user_id).await
    }

    #[tracing::instrument(skip(self, filter), fields(
        user_id = %user_id,
        per_page = filter.per_page
    ))]
    async fn find_all(&self, filter: TaskFilter, user_id: UserId) -> DomainResult<Vec<Task>> {
        self.find_all_impl(filter, user_id).await
    }

    #[tracing::instrument(skip(self, task), fields(task_id = %task.id, status = ?task.status))]
    async fn save(&self, task: &Task) -> DomainResult<()> {
        self.save_impl(task).await
    }
}
