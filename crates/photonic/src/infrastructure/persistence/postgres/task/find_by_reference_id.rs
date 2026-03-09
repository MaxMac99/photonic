use tracing::debug;
use uuid::Uuid;

use crate::domain::task::TaskType;
use crate::infrastructure::persistence::postgres::task::task_types::TaskTypeDb;
use crate::{
    domain::{error::DomainResult, task::Task, user::UserId},
    infrastructure::persistence::postgres::task::{
        entity::TaskDb, task_types::TaskStatusDb, PostgresTaskRepository,
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
        .await?;

        match &task {
            Some(_) => debug!("Task found"),
            None => debug!("Task not found"),
        }

        Ok(task.map(Into::into))
    }
}
