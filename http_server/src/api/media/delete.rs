use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use bson::oid::ObjectId;

pub async fn delete_medium(
    State(service): State<Arc<fotonic::Service>>,
    Path(medium_id): Path<ObjectId>,
) -> crate::error::Result<StatusCode> {
    service.move_to_trash(medium_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
