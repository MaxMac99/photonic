use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue},
};
use jwt_authorizer::JwtClaims;
use serde::Deserialize;
use snafu::{ResultExt, Whatever};
use tokio::fs;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use photonic::{model::FileItem, service::GetMediumFileType};

use crate::{api::user::User, AppState, error::Result};

#[derive(Debug, Clone, Deserialize)]
pub enum MediumItemGetFormat {
    #[serde(rename = "originals")]
    Originals,
    #[serde(rename = "edits")]
    Edits,
    #[serde(rename = "previews")]
    Previews,
    #[serde(rename = "sidecars")]
    Sidecars,
}

pub async fn get_medium_item_raw(
    State(AppState { service, .. }): State<AppState>,
    JwtClaims(user): JwtClaims<User>,
    Path((medium_id, format, item_id)): Path<(Uuid, MediumItemGetFormat, Uuid)>,
) -> Result<(HeaderMap, Body)> {
    let sub_item_type = match format {
        MediumItemGetFormat::Originals => GetMediumFileType::Original,
        MediumItemGetFormat::Edits => GetMediumFileType::Edit,
        MediumItemGetFormat::Previews => GetMediumFileType::Preview,
        MediumItemGetFormat::Sidecars => GetMediumFileType::Sidecar,
    };

    let original = service
        .get_medium_file(user.sub, medium_id, item_id, sub_item_type)
        .await?;

    stream_file_item(original).await
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
