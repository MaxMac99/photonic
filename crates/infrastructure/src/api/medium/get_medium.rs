use application::medium::queries::FindMediumQuery;
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::JwtClaims;
use tracing::{info, instrument};
use uuid::Uuid;

use super::dto::MediumDetailResponse;
use crate::{
    api::{error::ApiResult, state::AppState},
    auth::JwtUserClaims,
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}",
    tag = "medium",
    responses(
        (status = 200, content_type = "application/json", description = "Gets a single medium by ID", body = MediumDetailResponse),
    ),
)]
pub async fn get_medium(
    State(state): State<AppState>,
    Path(medium_id): Path<Uuid>,
    JwtClaims(claims): JwtClaims<JwtUserClaims>,
) -> ApiResult<(StatusCode, Json<MediumDetailResponse>)> {
    let user_id = claims.user_id();

    info!(
        user_id = %user_id,
        medium_id = %medium_id,
        "Fetching medium for user"
    );

    let query = FindMediumQuery { user_id, medium_id };

    let medium = state.medium_handlers.find_medium.handle(query).await?;
    let response: MediumDetailResponse = medium.into();

    info!(
        user_id = %user_id,
        medium_id = %medium_id,
        "Media retrieved successfully"
    );

    Ok((StatusCode::OK, Json(response)))
}
