use std::sync::Arc;

use axum::{Json, TypedHeader};
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::Result;

use crate::api::medium::model::input::CreateMediumInput;

pub async fn create_medium(
    State(service): State<Arc<core::Service>>,
    content_type: TypedHeader<ContentType>,
    opts: Query<CreateMediumInput>,
    body: Bytes,
) -> Result<(StatusCode, Json<String>)> {
    let opts = opts.0;
    let create_medium = core::CreateMediumInput {
        album_id: opts.album_id,
        filename: opts.filename,
        tags: opts.tags,
        date_taken: opts.date_taken,
        mime: content_type.0.into(),
    };
    let id = service.create_medium(create_medium, body.as_ref()).await?;

    Ok((StatusCode::CREATED, Json(id.to_hex())))
}
