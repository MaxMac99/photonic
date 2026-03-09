use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::JwtClaims;
use tracing::{info, instrument};
use uuid::Uuid;

use super::dto::MediumMetadataDto;
use crate::{
    application::{error::ApplicationResult, metadata::queries::FindMetadataByMediumIdQuery},
    infrastructure::{api::state::AppState, auth::JwtUserClaims},
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}/metadata",
    tag = "medium",
    responses(
        (status = 200, content_type = "application/json", description = "Gets metadata for a medium", body = MediumMetadataDto),
        (status = 404, description = "Metadata not found"),
    ),
)]
pub async fn get_medium_metadata(
    State(state): State<AppState>,
    Path(medium_id): Path<Uuid>,
    JwtClaims(claims): JwtClaims<JwtUserClaims>,
) -> ApplicationResult<(StatusCode, Json<MediumMetadataDto>)> {
    let user_id = claims.user_id();

    info!(
        user_id = %user_id,
        medium_id = %medium_id,
        "Fetching metadata for medium"
    );

    let query = FindMetadataByMediumIdQuery { medium_id, user_id };

    let metadata = state
        .metadata_handlers
        .find_metadata_by_medium_id
        .handle(query)
        .await?;

    let response: MediumMetadataDto = (&metadata).into();

    info!(
        user_id = %user_id,
        medium_id = %medium_id,
        "Metadata retrieved successfully"
    );

    Ok((StatusCode::OK, Json(response)))
}