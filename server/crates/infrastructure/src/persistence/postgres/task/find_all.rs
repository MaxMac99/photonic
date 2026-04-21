use domain::{
    error::DomainResult,
    shared::SortDirection,
    task::{Task, TaskFilter},
    user::UserId,
};
use sqlx::QueryBuilder;
use tracing::{debug, info};

use crate::persistence::postgres::{
    repo_error,
    task::{
        entity::TaskDb,
        task_types::{TaskStatusDb, TaskTypeDb},
        PostgresTaskRepository,
    },
};

impl PostgresTaskRepository {
    pub(super) async fn find_all_impl(
        &self,
        filter: TaskFilter,
        user_id: UserId,
    ) -> DomainResult<Vec<Task>> {
        debug!("Querying all tasks with filters");

        let direction_sql = match filter.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };

        let comparison_op = match filter.direction {
            SortDirection::Ascending => ">",
            SortDirection::Descending => "<",
        };

        let mut query = QueryBuilder::new(
            r#"
            SELECT id, reference_id, user_id, task_type,
                   status, error, created_at, started_at, completed_at
            FROM tasks
            WHERE user_id = "#,
        );

        query.push_bind(user_id);

        if let Some(reference_id) = filter.reference_id {
            query.push(" AND reference_id = ");
            query.push_bind(reference_id);
        }

        if !filter.task_types.is_empty() {
            query.push(" AND task_type IN (");
            let mut separated = query.separated(", ");
            for task_type in &filter.task_types {
                let task_type = TaskTypeDb::from(*task_type);
                separated.push_bind(task_type);
            }
            query.push(") ");
        }

        if let Some(status) = filter.status {
            query.push(" AND status = ");
            query.push_bind(TaskStatusDb::from(status));
        }

        if let Some(start_date) = filter.start_date {
            query.push(" AND created_at >= ");
            query.push_bind(start_date);
        }
        if let Some(end_date) = filter.end_date {
            query.push(" AND created_at <= ");
            query.push_bind(end_date);
        }

        if let Some(cursor) = &filter.cursor {
            query.push(" AND (created_at, id) ");
            query.push(comparison_op);
            query.push(" (");
            query.push_bind(cursor.last_date);
            query.push(", ");
            query.push_bind(cursor.last_id);
            query.push(") ");
        }

        query.push(" ORDER BY created_at ");
        query.push(direction_sql);
        query.push(", id ");
        query.push(direction_sql);
        query.push(" LIMIT ");
        query.push_bind(filter.per_page as i64);

        let tasks: Vec<TaskDb> = query
            .build_query_as::<TaskDb>()
            .fetch_all(&self.pool)
            .await
            .map_err(repo_error)?;

        info!(count = tasks.len(), "Tasks retrieved");

        Ok(tasks.into_iter().map(Into::into).collect())
    }
}
