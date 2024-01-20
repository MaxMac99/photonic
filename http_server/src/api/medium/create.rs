use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Result;
use axum_extra::headers::ContentType;
use axum_extra::TypedHeader;
use futures::TryFutureExt;
use tokio::fs;
use tracing::log::info;

use crate::api::medium::model::create_medium::CreateMediumInput;
use crate::ResponseError;

pub async fn create_medium(
    State(service): State<Arc<fotonic::Service>>,
    content_type: TypedHeader<ContentType>,
    opts: Query<CreateMediumInput>,
    body: Body,
) -> Result<(StatusCode, Json<String>)> {
    let opts = opts.0;

    let temp_path = service.store_stream_temporarily(&opts.extension, body.into_data_stream())
        .await
        .map_err(ResponseError::from)?;

    let create_medium = fotonic::service::CreateMediumInput {
        album_id: opts.album_id,
        filename: opts.filename,
        extension: opts.extension,
        tags: opts.tags,
        date_taken: opts.date_taken,
        mime: content_type.0.into(),
    };
    let id = service.create_medium(create_medium, &temp_path)
        .or_else(|err| async {
            // Try remove temporary file if it could not be stored
            let _ = fs::remove_file(&temp_path).await;
            Err(err)
        })
        .await
        .map_err(ResponseError::from)?;

    info!("Successfully uploaded file with id {}", &id);
    Ok((StatusCode::CREATED, Json(id.to_hex())))
}
