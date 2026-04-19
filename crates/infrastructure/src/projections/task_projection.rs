use async_trait::async_trait;
use domain::task::events::{
    TaskCompletedEvent, TaskCreatedEvent, TaskFailedEvent, TaskStartedEvent,
};
use event_sourcing::{
    error::{EventSourcingError, Result},
    projection::handler::ProjectionHandler,
};
use sqlx::{Postgres, Transaction};
use tracing::info;

use super::{register_event, RegisterProjection};
use crate::persistence::postgres::task::task_types::TaskTypeDb;

/// Projection that maintains the tasks read model table.
pub struct TaskProjection;

impl TaskProjection {
    pub fn new() -> Self {
        Self
    }
}

impl RegisterProjection for TaskProjection {
    fn register(
        bus: &super::PgProjectionBus,
        registry: &mut super::EventTypeRegistry,
    ) -> Result<()> {
        register_event::<TaskCreatedEvent, _>(bus, registry, Self::new())?;
        register_event::<TaskStartedEvent, _>(bus, registry, Self::new())?;
        register_event::<TaskCompletedEvent, _>(bus, registry, Self::new())?;
        register_event::<TaskFailedEvent, _>(bus, registry, Self::new())?;
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<TaskCreatedEvent, i64, Transaction<'static, Postgres>> for TaskProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &TaskCreatedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        let task_type_db = TaskTypeDb::from(event.task_type);

        sqlx::query(
            "INSERT INTO tasks (id, task_type, reference_id, user_id, status, created_at) \
             VALUES ($1, $2, $3, $4, 'pending', NOW()) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(event.task_id)
        .bind(task_type_db as TaskTypeDb)
        .bind(event.reference_id)
        .bind(event.user_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to insert task: {}", e),
        })?;

        info!(task_id = %event.task_id, "TaskProjection: task created");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<TaskStartedEvent, i64, Transaction<'static, Postgres>> for TaskProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &TaskStartedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query("UPDATE tasks SET status = 'in_progress', started_at = $2 WHERE id = $1")
            .bind(event.task_id)
            .bind(event.metadata.occurred_at)
            .execute(&mut **tx)
            .await
            .map_err(|e| EventSourcingError::Projection {
                message: format!("Failed to update task to in_progress: {}", e),
            })?;

        info!(task_id = %event.task_id, "TaskProjection: task started");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<TaskCompletedEvent, i64, Transaction<'static, Postgres>> for TaskProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &TaskCompletedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query("UPDATE tasks SET status = 'completed', completed_at = $2 WHERE id = $1")
            .bind(event.task_id)
            .bind(event.metadata.occurred_at)
            .execute(&mut **tx)
            .await
            .map_err(|e| EventSourcingError::Projection {
                message: format!("Failed to update task to completed: {}", e),
            })?;

        info!(task_id = %event.task_id, "TaskProjection: task completed");
        Ok(())
    }
}

#[async_trait]
impl ProjectionHandler<TaskFailedEvent, i64, Transaction<'static, Postgres>> for TaskProjection {
    type Error = event_sourcing::error::EventSourcingError;

    async fn handle(
        &self,
        event: &TaskFailedEvent,
        _sequence: i64,
        tx: &mut Transaction<'static, Postgres>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE tasks SET status = 'failed', error = $2, completed_at = $3 WHERE id = $1",
        )
        .bind(event.task_id)
        .bind(&event.error)
        .bind(event.metadata.occurred_at)
        .execute(&mut **tx)
        .await
        .map_err(|e| EventSourcingError::Projection {
            message: format!("Failed to update task to failed: {}", e),
        })?;

        info!(task_id = %event.task_id, "TaskProjection: task failed");
        Ok(())
    }
}
