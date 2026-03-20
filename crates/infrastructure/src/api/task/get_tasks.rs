use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::JwtClaims;
use tracing::instrument;

use super::dto::{FindTasksOptions, TaskListResponse};
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

    Ok((StatusCode::OK, Json(vec![])))
}
