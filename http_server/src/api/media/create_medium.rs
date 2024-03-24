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

use crate::{api::user::User, AppState, error::Result};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMediumInput {
    pub album_id: Option<Uuid>,
    pub filename: String,
    pub extension: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

pub async fn create_medium(
    State(AppState { service, .. }): State<AppState>,
    content_type: TypedHeader<ContentType>,
    opts: Query<CreateMediumInput>,
    JwtClaims(user): JwtClaims<User>,
    body: Body,
) -> Result<(StatusCode, Json<String>)> {
    service.create_or_update_user(user.clone().into()).await?;

    let opts = opts.0;

    let temp_path = service
        .store_stream_temporarily(&opts.extension, body.into_data_stream())
        .await?;

    let create_medium = fotonic::service::CreateMediumInput {
        album_id: opts.album_id,
        filename: opts.filename,
        extension: opts.extension,
        tags: opts.tags,
        date_taken: opts.date_taken,
        mime: content_type.0.into(),
    };
    let id = service
        .create_medium(
            user.sub,
            user.get_username().unwrap(),
            create_medium,
            &temp_path,
        )
        .or_else(|err| async {
            // Try remove temporary file if it could not be stored
            if let Err(remove_err) = fs::remove_file(&temp_path).await {
                error!("Could not delete file for rollback: {}", remove_err);
            }
            Err(err)
        })
        .await?;

    info!("Successfully uploaded file with id {}", &id);
    Ok((StatusCode::CREATED, Json(id.to_string())))
}
