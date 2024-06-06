use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::{headers::ContentType, TypedHeader};
use chrono::{DateTime, FixedOffset};
use futures::TryFutureExt;
use jwt_authorizer::JwtClaims;
use serde::Deserialize;
use tokio::fs;
use tracing::{error, log::info};
use uuid::Uuid;

use crate::{api::user::User, error::Result, AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMediumInput {
    pub album_id: Option<Uuid>,
    pub filename: String,
    pub extension: String,
    #[serde(default = "default_prio")]
    pub priority: i32,
    #[serde(default)]
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

fn default_prio() -> i32 {
    10
}

pub async fn create_medium(
    State(AppState { service, .. }): State<AppState>,
    content_type: TypedHeader<ContentType>,
    opts: Query<CreateMediumInput>,
    JwtClaims(user): JwtClaims<User>,
    body: Body,
) -> Result<(StatusCode, Json<String>)> {
    let opts = opts.0;
    let create_medium = photonic::service::CreateMediumInput {
        user: user.into(),
        album_id: opts.album_id,
        filename: opts.filename,
        extension: opts.extension,
        tags: opts.tags,
        date_taken: opts.date_taken,
        mime: content_type.0.into(),
        priority: opts.priority,
    };
    let id = service
        .create_medium_from_stream(create_medium, body.into_data_stream())
        .await?;

    info!("Successfully uploaded file with id {}", &id);
    Ok((StatusCode::CREATED, Json(id.to_string())))
}
