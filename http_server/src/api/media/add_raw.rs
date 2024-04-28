use axum::{
    body::Body,
    extract::{Path, Query, State},
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

use photonic::service::AddMediumItemType;

use crate::{
    api::{media::create_medium::CreateMediumInput, user::User},
    AppState,
};

#[derive(Debug, Clone, Deserialize)]
pub struct AddRawInput {
    pub filename: String,
    pub extension: String,
    #[serde(default = "default_prio")]
    pub priority: i32,
    pub date_taken: Option<DateTime<FixedOffset>>,
}

fn default_prio() -> i32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub enum MediumItemFormat {
    Originals,
    Edits,
    Previews,
    Sidecars,
}

pub async fn add_raw(
    State(AppState { service, .. }): State<AppState>,
    content_type: TypedHeader<ContentType>,
    Path((medium_id, format)): Path<(Uuid, MediumItemFormat)>,
    opts: Query<AddRawInput>,
    JwtClaims(user): JwtClaims<User>,
    body: Body,
) -> crate::error::Result<(StatusCode, Json<String>)> {
    service.create_or_update_user(user.clone().into()).await?;

    let opts = opts.0;

    let temp_path = service
        .store_stream_temporarily(&opts.extension, body.into_data_stream())
        .await?;

    let input = photonic::service::AddMediumItemInput {
        user_id: user.sub,
        username: user.get_username().unwrap(),
        item_type: match format {
            MediumItemFormat::Originals => AddMediumItemType::Original,
            MediumItemFormat::Edits => AddMediumItemType::Edit,
            MediumItemFormat::Previews => AddMediumItemType::Preview,
            MediumItemFormat::Sidecars => AddMediumItemType::Sidecar,
        },
        medium_id,
        filename: opts.filename,
        extension: opts.extension,
        date_taken: opts.date_taken,
        mime: content_type.0.into(),
        priority: opts.priority,
    };
    let id = service
        .add_raw_file(input, &temp_path)
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
