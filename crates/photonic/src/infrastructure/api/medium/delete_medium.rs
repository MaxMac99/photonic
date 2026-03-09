use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
};
use jwt_authorizer::JwtClaims;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    application::error::ApplicationResult,
    infrastructure::{api::state::AppState, auth::JwtUserClaims},
};

#[instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    delete,
    path = "/{medium_id}",
    tag = "medium",
    responses(
        (status = 204, description = "Deletes the medium"),
    ),
    params(
        ("medium_id" = Uuid, Path, description = "The id of the medium to delete"),
    ),
)]
pub async fn delete_medium(
    State(state): State<AppState>,
    Path(medium_id): Path<Uuid>,
    JwtClaims(user): JwtClaims<JwtUserClaims>,
) -> ApplicationResult<StatusCode> {
    todo!()
}
