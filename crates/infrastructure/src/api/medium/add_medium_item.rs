use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::{
    headers::{ContentLength, ContentType},
    TypedHeader,
};
use jwt_authorizer::JwtClaims;
use tracing::instrument;
use uuid::Uuid;

use super::dto::{CreateMediumItemInput, MediumItemTypeDto};
use crate::{
    api::{error::ApiResult, router::Binary, state::AppState},
    auth::JwtUserClaims,
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    post,
    path = "/{medium_id}/item/{format}",
    tag = "medium",
    request_body(
        content = Binary,
        content_type = "*/*"
    ),
    responses(
        (status = 201, content_type = "application/json", description = "The id of the new medium item", body = Uuid),
    ),
    params(CreateMediumItemInput),
)]
pub async fn add_medium_item(
    State(state): State<AppState>,
    Path((medium_id, format)): Path<(Uuid, MediumItemTypeDto)>,
    content_length: TypedHeader<ContentLength>,
    content_type: TypedHeader<ContentType>,
    Query(medium_item_opts): Query<CreateMediumItemInput>,
    JwtClaims(user): JwtClaims<JwtUserClaims>,
    body: Body,
) -> ApiResult<(StatusCode, Json<Uuid>)> {
    todo!()
}
