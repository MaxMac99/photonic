use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use jwt_authorizer::JwtClaims;
use tracing::instrument;
use uuid::Uuid;

use super::dto::GetMediumPreviewOptions;
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
    todo!()
}
