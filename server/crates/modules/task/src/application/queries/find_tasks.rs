use std::sync::Arc;

use derive_new::new;
use kernel::{ApplicationResult, UserId};
use tracing::{debug, error, info, instrument};

use crate::{
    application::queries::TaskQueryPort,
    domain::{Task, TaskFilter},
};

#[derive(Debug)]
pub struct FindTasksQuery {
    pub user_id: UserId,
    pub filter: TaskFilter,
}

#[derive(new)]
pub struct FindTasksHandler {
    query_port: Arc<dyn TaskQueryPort>,
}

impl FindTasksHandler {
    #[instrument(skip(self), fields(
        user_id = %query.user_id,
        per_page = query.filter.per_page,
        has_cursor = query.filter.cursor.is_some(),
    ))]
    pub async fn handle(&self, query: FindTasksQuery) -> ApplicationResult<Vec<Task>> {
        info!("Finding tasks for user");

        let tasks = self
            .query_port
            .find_all(query.filter, query.user_id)
            .await
            .inspect_err(|e| {
                error!(error = %e, "Failed to find tasks");
            })?;

        debug!(count = tasks.len(), "Tasks retrieved successfully");

        Ok(tasks)
    }
}
