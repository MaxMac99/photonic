use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::{
    headers::{ContentLength, ContentType},
    TypedHeader,
};
use jwt_authorizer::JwtClaims;
use medium::{
    application::commands::AddMediumItemCommand,
    domain::{Dimensions, MediumItemType, ThumbnailVariant},
};
use tracing::{error, info, instrument};
use uuid::Uuid;

use super::dto::{AddMediumItemInput, MediumItemTypeDto};
use crate::{
    api::{
        error::{ApiError, ApiResult},
        router::Binary,
        state::AppState,
    },
    auth::JwtUserClaims,
};

/// Memory cap for buffered uploads via this endpoint. Thumbnails are tiny;
/// non-preview items (edits, sidecars) get a generous but bounded allowance.
const MAX_ITEM_BODY_SIZE: usize = 100 * 1024 * 1024;

#[instrument(skip(state, body))]
#[debug_handler]
#[utoipa::path(
    post,
    path = "/{medium_id}/item/{format}",
    tag = "medium",
    request_body(
        content = Binary,
        content_type = "*/*"
    ),
    responses(
        (status = 201, content_type = "application/json", description = "The id of the new medium item", body = Uuid),
    ),
    params(AddMediumItemInput),
)]
pub async fn add_medium_item(
    State(state): State<AppState>,
    Path((medium_id, format)): Path<(Uuid, MediumItemTypeDto)>,
    content_length: TypedHeader<ContentLength>,
    content_type: TypedHeader<ContentType>,
    Query(medium_item_opts): Query<AddMediumItemInput>,
    JwtClaims(user): JwtClaims<JwtUserClaims>,
    body: Body,
) -> ApiResult<(StatusCode, Json<Uuid>)> {
    let user_id = user.user_id();
    let item_type: MediumItemType = format.into();

    info!(
        user_id = %user_id,
        medium_id = %medium_id,
        item_type = ?item_type,
        file_size = content_length.0.0,
        mime_type = %content_type.0,
        "Medium item upload initiated"
    );

    let data = read_body(body).await?;

    let command = AddMediumItemCommand {
        user_id,
        medium_id,
        item_type,
        variant: medium_item_opts.variant.map(ThumbnailVariant::from),
        mime_type: content_type.0.into(),
        filename: medium_item_opts.filename,
        filesize: content_length.0 .0.into(),
        priority: Some(medium_item_opts.priority),
        dimensions: match (medium_item_opts.width, medium_item_opts.height) {
            (Some(w), Some(h)) => Dimensions::new(w, h).ok(),
            _ => None,
        },
        data,
    };

    match state.medium_handlers.add_medium_item.handle(command).await {
        Ok(item_id) => {
            info!(
                user_id = %user_id,
                medium_id = %medium_id,
                item_id = %item_id,
                "Medium item added successfully"
            );
            Ok((StatusCode::CREATED, Json(item_id)))
        }
        Err(e) => {
            error!(
                user_id = %user_id,
                medium_id = %medium_id,
                error = %kernel::app_error::format_error_with_backtrace(&e),
                "Failed to add medium item"
            );
            Err(e.into())
        }
    }
}

/// Buffers the request body in memory, bounded by [`MAX_ITEM_BODY_SIZE`].
/// Large-file streaming uploads go through `create_medium` instead; this
/// endpoint serves thumbnails and small companion items.
async fn read_body(body: Body) -> ApiResult<Vec<u8>> {
    axum::body::to_bytes(body, MAX_ITEM_BODY_SIZE)
        .await
        .map_err(|e| {
            ApiError(kernel::app_error::ApplicationError::Domain {
                source: kernel::error::ValidationSnafu {
                    message: format!("Failed to read request body: {e}"),
                }
                .build(),
            })
        })
        .map(|bytes| bytes.to_vec())
}
