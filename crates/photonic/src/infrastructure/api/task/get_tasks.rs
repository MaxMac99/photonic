use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::JwtClaims;
use tracing::{info, instrument};

use super::dto::{DirectionDto, FindTasksOptions, MediumListResponse, TaskListResponse};
use crate::{
    application::{error::ApplicationResult, medium::queries::FindAllMediaQuery},
    domain::medium::MediumFilter,
    infrastructure::{api::state::AppState, auth::JwtUserClaims},
    shared::{KeysetCursor, SortDirection},
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    tag = "tasks",
    responses(
        (status = 200, content_type = "application/json", description = "Gets all media. Can be filtered by date", body = [MediumListResponse]),
    ),
    params(FindTasksOptions),
)]
pub async fn get_tasks(
    State(state): State<AppState>,
    Query(find_tasks_opts): Query<FindTasksOptions>,
    JwtClaims(claims): JwtClaims<JwtUserClaims>,
) -> ApplicationResult<(StatusCode, Json<Vec<TaskListResponse>>)> {
    let user_id = claims.user_id();

    info!(
        user_id = %user_id,
        per_page = find_tasks_opts.per_page,
        has_date_filter = find_tasks_opts.start_date.is_some() || find_all_media_opts.end_date.is_some(),
        "Fetching all media for user"
    );

    let filter = TaskFilter::new(
        find_all_media_opts.start_date,
        find_all_media_opts.end_date,
        Some(find_all_media_opts.per_page),
        match (
            find_all_media_opts.page_last_date,
            find_all_media_opts.page_last_id,
        ) {
            (Some(date), Some(id)) => Some(KeysetCursor::new(date, id)),
            _ => None,
        },
        find_all_media_opts.tags,
        find_all_media_opts.album_id,
        Some(match find_all_media_opts.direction {
            DirectionDto::Asc => SortDirection::Ascending,
            DirectionDto::Desc => SortDirection::Descending,
        }),
        find_all_media_opts.include_no_album,
    )?;

    let query = FindAllMediaQuery { user_id, filter };

    let media = state.medium_handlers.find_all_media.handle(query).await?;

    let responses: Vec<MediumListResponse> = media.iter().map(|m| m.into()).collect();

    info!(
        user_id = %user_id,
        count = responses.len(),
        "Media retrieved successfully"
    );

    Ok((StatusCode::OK, Json(responses)))
}
