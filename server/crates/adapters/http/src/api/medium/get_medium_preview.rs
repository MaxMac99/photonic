use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use jwt_authorizer::JwtClaims;
use medium::application::queries::GetMediumPreviewFileQuery;
use tracing::{error, info, instrument};
use uuid::Uuid;

use super::{dto::GetMediumPreviewOptions, get_medium_item::file_to_response};
use crate::{
    api::{error::ApiResult, router::Binary, state::AppState},
    auth::JwtUserClaims,
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}/preview",
    tag = "medium",
    responses(
        (status = 200, description = "The raw file", body = Binary, content_type = "*/*", headers(
            ("content-type" = String)
        )),
    ),
    params(GetMediumPreviewOptions),
)]
pub async fn get_medium_preview(
    State(state): State<AppState>,
    Path(medium_id): Path<Uuid>,
    Query(opts): Query<GetMediumPreviewOptions>,
    JwtClaims(user): JwtClaims<JwtUserClaims>,
) -> ApiResult<(StatusCode, HeaderMap, Body)> {
    let user_id = user.user_id();
    info!(user_id = %user_id, medium_id = %medium_id, "Fetching medium preview");

    let requested_edge = opts
        .width
        .filter(|w| *w > 0)
        .max(opts.height.filter(|h| *h > 0))
        .map(|edge| edge as u32);

    let query = GetMediumPreviewFileQuery {
        user_id,
        medium_id,
        requested_edge,
    };

    match state
        .medium_handlers
        .get_medium_file
        .get_preview_file(query)
        .await
    {
        Ok(file) => Ok(file_to_response(file)),
        Err(e) => {
            error!(
                error = %kernel::app_error::format_error_with_backtrace(&e),
                "Failed to retrieve medium preview"
            );
            Err(e.into())
        }
    }
}
