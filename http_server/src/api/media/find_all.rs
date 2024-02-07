use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::JwtClaims;

use fotonic::service::FindAllMediaInput;

use crate::{
    api::{media::model::MediumOverview, user::User},
    error::Result,
};

pub async fn find_all(
    State(service): State<Arc<fotonic::Service>>,
    JwtClaims(user): JwtClaims<User>,
    opts: Query<FindAllMediaInput>,
) -> Result<(StatusCode, Json<Vec<MediumOverview>>)> {
    let media: Vec<MediumOverview> = service
        .find_all_media(user.sub, &opts)
        .await?
        .into_iter()
        .map(MediumOverview::from)
        .collect();
    Ok((StatusCode::OK, Json(media)))
}
