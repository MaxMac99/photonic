use domain::{
    error::DomainResult,
    task::{Task, TaskType},
    user::UserId,
};
use tracing::debug;
use uuid::Uuid;

use crate::persistence::postgres::{
    repo_error,
    task::{
        entity::TaskDb,
        task_types::{TaskStatusDb, TaskTypeDb},
        PostgresTaskRepository,
    },
};

impl PostgresTaskRepository {
    pub(super) async fn find_by_reference_id_impl(
        &self,
        reference_id: Uuid,
        task_type: TaskType,
        user_id: UserId,
    ) -> DomainResult<Option<Task>> {
        debug!("Querying task by id");

        let task_type = TaskTypeDb::from(task_type);
        let task = sqlx::query_as!(
            TaskDb,
            r#"
            SELECT
                id,
                reference_id,
                user_id,
                task_type as "task_type: TaskTypeDb",
                status as "status: TaskStatusDb",
                error,
                created_at,
                started_at,
                completed_at
            FROM tasks
            WHERE reference_id = $1 AND task_type = $2 AND user_id = $3
            "#,
            reference_id,
            task_type as TaskTypeDb,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(repo_error)?;

        match &task {
            Some(_) => debug!("Task found"),
            None => debug!("Task not found"),
        }

        Ok(task.map(Into::into))
    }
}
