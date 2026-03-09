use tracing::{debug, info};

use crate::infrastructure::persistence::postgres::task::task_types::TaskTypeDb;
use crate::{
    domain::{error::DomainResult, task::Task},
    infrastructure::persistence::postgres::task::{
        entity::TaskDb, task_types::TaskStatusDb, PostgresTaskRepository,
    },
};

impl PostgresTaskRepository {
    pub(super) async fn save_impl(&self, task: &Task) -> DomainResult<()> {
        debug!("Saving task (upsert)");

        let task_db = TaskDb::from(task.clone());

        // Use conditional update to respect status ordering:
        // pending < in_progress < completed/failed
        // Never overwrite a more advanced status with an earlier one.
        sqlx::query!(
            r#"INSERT INTO tasks (
                id,
                reference_id,
                user_id,
                task_type,
                status,
                error,
                created_at,
                started_at,
                completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE
            SET status = CASE
                    -- Only update status if transitioning forward in the state machine
                    WHEN tasks.status = 'pending' THEN EXCLUDED.status
                    WHEN tasks.status = 'in_progress' AND EXCLUDED.status IN ('completed', 'failed') THEN EXCLUDED.status
                    ELSE tasks.status
                END,
                error = CASE
                    WHEN tasks.status = 'pending' THEN EXCLUDED.error
                    WHEN tasks.status = 'in_progress' AND EXCLUDED.status IN ('completed', 'failed') THEN EXCLUDED.error
                    ELSE tasks.error
                END,
                started_at = COALESCE(tasks.started_at, EXCLUDED.started_at),
                completed_at = COALESCE(tasks.completed_at, EXCLUDED.completed_at)
            "#,
            task_db.id,
            task_db.reference_id,
            task_db.user_id,
            task_db.task_type as TaskTypeDb,
            task_db.status as TaskStatusDb,
            task_db.error,
            task_db.created_at,
            task_db.started_at,
            task_db.completed_at,
        )
        .execute(&self.pool)
        .await?;

        info!(task_id = %task.id, "Task saved successfully");

        Ok(())
    }
}
