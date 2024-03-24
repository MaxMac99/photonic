use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::AppState;

pub async fn delete_medium(
    State(AppState { service, .. }): State<AppState>,
    Path(medium_id): Path<Uuid>,
) -> crate::error::Result<StatusCode> {
    service.move_to_trash(medium_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
