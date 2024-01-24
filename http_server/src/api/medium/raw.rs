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

use crate::error::Result;

pub async fn get_medium_original_raw(
    State(service): State<Arc<fotonic::Service>>,
    Path((medium_id, originals_id)): Path<(ObjectId, ObjectId)>,
) -> Result<(HeaderMap, Body)> {
    let original = service.get_medium_original(medium_id, originals_id).await?;

    let mut headers = HeaderMap::new();
    headers.append(
        header::CONTENT_TYPE,
        HeaderValue::from_str(original.mime.as_ref())
            .whatever_context::<&str, Whatever>("Could not parse header")?,
    );

    let file = fs::File::open(&original.path)
        .await
        .with_whatever_context::<_, String, Whatever>(|err| {
            format!("Could not open file {:?}: {:?}", original.path, err)
        })?;
    let stream = ReaderStream::new(file);

    Ok((headers, Body::from_stream(stream)))
}
