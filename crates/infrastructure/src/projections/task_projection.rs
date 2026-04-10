use std::borrow::Cow;

use application::error::ApplicationResult;
use application::projection::Projection;
use async_trait::async_trait;
use domain::task::events::TaskEvent;
use sqlx::PgPool;
use tracing::debug;

/// Projection that maintains the tasks read model table.
pub struct TaskProjection {
    pool: PgPool,
}

impl TaskProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projection<TaskEvent> for TaskProjection {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("task_read_model")
    }

    async fn handle(&self, event: &TaskEvent, global_sequence: i64) -> ApplicationResult<()> {
        debug!(
            global_sequence = global_sequence,
            "TaskProjection handling event"
        );

        match event {
            TaskEvent::TaskCreated(_e) => {
                // TODO: INSERT INTO tasks
            }
            TaskEvent::TaskStarted(_e) => {
                // TODO: UPDATE tasks SET status = 'in_progress', started_at
            }
            TaskEvent::TaskCompleted(_e) => {
                // TODO: UPDATE tasks SET status = 'completed', completed_at
            }
            TaskEvent::TaskFailed(_e) => {
                // TODO: UPDATE tasks SET status = 'failed', error, completed_at
            }
        }

        Ok(())
    }
}
