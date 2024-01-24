use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue},
};
use bson::oid::ObjectId;
use snafu::{ResultExt, Whatever};
use tokio::fs;
use tokio_util::io::ReaderStream;

use fotonic::{model::FileItem, service::MediumFileSubItem};

use crate::error::Result;

pub async fn get_medium_original_raw(
    State(service): State<Arc<fotonic::Service>>,
    Path((medium_id, item_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(HeaderMap, Body)> {
    let original = service
        .get_medium_file(medium_id, MediumFileSubItem::Original(item_id))
        .await?;

    stream_file_item(original).await
}

pub async fn get_medium_edit_raw(
    State(service): State<Arc<fotonic::Service>>,
    Path((medium_id, item_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(HeaderMap, Body)> {
    let edit = service
        .get_medium_file(medium_id, MediumFileSubItem::Edit(item_id))
        .await?;

    stream_file_item(edit).await
}

pub async fn get_medium_preview_raw(
    State(service): State<Arc<fotonic::Service>>,
    Path(medium_id): Path<ObjectId>,
) -> Result<(HeaderMap, Body)> {
    let edit = service
        .get_medium_file(medium_id, MediumFileSubItem::Preview)
        .await?;

    stream_file_item(edit).await
}

pub async fn get_medium_sidecar_raw(
    State(service): State<Arc<fotonic::Service>>,
    Path((medium_id, item_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(HeaderMap, Body)> {
    let edit = service
        .get_medium_file(medium_id, MediumFileSubItem::Sidecar(item_id))
        .await?;

    stream_file_item(edit).await
}

async fn stream_file_item(file_item: FileItem) -> Result<(HeaderMap, Body)> {
    let mut headers = HeaderMap::new();
    headers.append(
        header::CONTENT_TYPE,
        HeaderValue::from_str(file_item.mime.as_ref())
            .whatever_context::<&str, Whatever>("Could not parse header")?,
    );

    let file = fs::File::open(&file_item.path)
        .await
        .with_whatever_context::<_, String, Whatever>(|err| {
            format!("Could not open file {:?}: {:?}", file_item.path, err)
        })?;
    let stream = ReaderStream::new(file);
    Ok((headers, Body::from_stream(stream)))
}
