use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use jwt_authorizer::JwtClaims;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    api::{error::ApiResult, router::Binary, state::AppState},
    auth::JwtUserClaims,
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}/item/{item_id}/raw",
    tag = "medium",
    responses(
        (status = 200, description = "The raw file", body = Binary, content_type = "*/*", headers(
            ("content-type" = String)
        )),
    ),
)]
pub async fn get_medium_item(
    State(state): State<AppState>,
    Path((medium_id, item_id)): Path<(Uuid, Uuid)>,
    JwtClaims(user): JwtClaims<JwtUserClaims>,
) -> ApiResult<(StatusCode, HeaderMap, Body)> {
    todo!()
}
