use std::sync::Arc;

use derive_new::new;
use domain::{
    error::format_error_with_backtrace,
    task::{Task, TaskFilter},
    user::UserId,
};
use tracing::{debug, error, info, instrument};

use crate::{error::ApplicationResult, task::TaskRepository};

#[allow(dead_code)]
#[derive(Debug)]
pub struct FindTasksQuery {
    pub user_id: UserId,
    pub filter: TaskFilter,
}

#[allow(dead_code)]
#[derive(new)]
pub struct FindTasksHandler {
    processing_repository: Arc<dyn TaskRepository>,
}

impl FindTasksHandler {
    #[allow(dead_code)]
    #[instrument(skip(self), fields(
        user_id = %query.user_id,
        per_page = query.filter.per_page,
        has_cursor = query.filter.cursor.is_some(),
    ))]
    pub async fn handle(&self, query: FindTasksQuery) -> ApplicationResult<Vec<Task>> {
        info!("Finding tasks for user");

        let tasks = self
            .processing_repository
            .find_all(query.filter, query.user_id)
            .await
            .inspect_err(|e| {
                error!(error = %format_error_with_backtrace(e), "Failed to find tasks");
            })?;

        debug!(count = tasks.len(), "Tasks retrieved successfully");

        Ok(tasks)
    }
}
