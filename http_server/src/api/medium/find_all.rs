use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Result,
    Json,
};

use fotonic::service::FindAllMediaInput;

use crate::{api::medium::model::MediumOverview, error::ResponseError};

pub async fn find_all(
    State(service): State<Arc<fotonic::Service>>,
    opts: Query<FindAllMediaInput>,
) -> Result<(StatusCode, Json<Vec<MediumOverview>>)> {
    let media: Vec<MediumOverview> = service
        .find_all_media(&opts)
        .await
        .map_err(ResponseError::from)?
        .into_iter()
        .map(MediumOverview::from)
        .collect();
    Ok((StatusCode::OK, Json(media)))
}
