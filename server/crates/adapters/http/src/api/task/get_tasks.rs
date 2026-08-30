use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::JwtClaims;
use kernel::shared::{KeysetCursor, SortDirection};
use task::{
    application::queries::FindTasksQuery,
    domain::{TaskFilter, TaskStatus},
};
use tracing::{info, instrument};

use super::dto::{DirectionDto, FindTasksOptions, TaskListResponse, TaskStatusDto};
use crate::{
    api::{error::ApiResult, state::AppState},
    auth::JwtUserClaims,
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    tag = "tasks",
    responses(
        (status = 200, content_type = "application/json", description = "Gets all tasks", body = [TaskListResponse]),
    ),
    params(FindTasksOptions),
)]
pub async fn get_tasks(
    State(state): State<AppState>,
    Query(find_tasks_opts): Query<FindTasksOptions>,
    JwtClaims(claims): JwtClaims<JwtUserClaims>,
) -> ApiResult<(StatusCode, Json<Vec<TaskListResponse>>)> {
    let user_id = claims.user_id();

    info!(
        user_id = %user_id,
        per_page = find_tasks_opts.per_page,
        has_date_filter = find_tasks_opts.start_date.is_some() || find_tasks_opts.end_date.is_some(),
        "Fetching tasks for user"
    );

    // The read model supports one status filter; the API accepts a list, so
    // the first requested state is applied.
    let status = find_tasks_opts.states.first().map(|state| match state {
        TaskStatusDto::Pending => TaskStatus::Pending,
        TaskStatusDto::InProgress => TaskStatus::InProgress,
        TaskStatusDto::Completed => TaskStatus::Completed,
        // The filter is a status label; the failure message is not part of
        // the filter key.
        TaskStatusDto::Failed => TaskStatus::Failed(String::new()),
    });

    let filter = TaskFilter {
        task_types: find_tasks_opts
            .types
            .iter()
            .map(|t| t.clone().into())
            .collect(),
        reference_id: None,
        status,
        start_date: find_tasks_opts.start_date,
        end_date: find_tasks_opts.end_date,
        per_page: find_tasks_opts.per_page,
        cursor: match (find_tasks_opts.page_last_date, find_tasks_opts.page_last_id) {
            (Some(date), Some(id)) => Some(KeysetCursor::new(date, id)),
            _ => None,
        },
        direction: match find_tasks_opts.direction {
            DirectionDto::Asc => SortDirection::Ascending,
            DirectionDto::Desc => SortDirection::Descending,
        },
    };

    let query = FindTasksQuery { user_id, filter };

    let tasks = state.processing_handlers.find_tasks.handle(query).await?;

    let responses: Vec<TaskListResponse> = tasks.into_iter().map(Into::into).collect();

    info!(
        user_id = %user_id,
        count = responses.len(),
        "Tasks retrieved successfully"
    );

    Ok((StatusCode::OK, Json(responses)))
}
