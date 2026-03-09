use axum::{
    body::Body,
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::{
    headers::{ContentLength, ContentType},
    TypedHeader,
};
use futures_util::TryStreamExt;
use jwt_authorizer::JwtClaims;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;
use tracing::{error, info, instrument};
use uuid::Uuid;

use super::dto::{CreateMediumInput, CreateMediumItemInput};
use crate::{
    application::{
        error::{format_error_with_backtrace, ApplicationResult},
        medium::commands::create_medium_stream::CreateMediumStreamCommand,
    },
    infrastructure::{
        api::{router::Binary, state::AppState},
        auth::JwtUserClaims,
    },
};

#[instrument(skip(state, body))]
#[debug_handler]
#[utoipa::path(
    post,
    path = "",
    tag = "medium",
    request_body(
        content = Binary,
        content_type = "*/*"
    ),
    responses(
        (status = 201, content_type = "application/json", description = "The id of the newly created medium", body = Uuid),
    ),
    params(CreateMediumInput, CreateMediumItemInput),
)]
pub async fn create_medium(
    State(state): State<AppState>,
    content_length: TypedHeader<ContentLength>,
    content_type: TypedHeader<ContentType>,
    Query(medium_opts): Query<CreateMediumInput>,
    Query(medium_item_opts): Query<CreateMediumItemInput>,
    JwtClaims(claims): JwtClaims<JwtUserClaims>,
    body: Body,
) -> ApplicationResult<(StatusCode, Json<Uuid>)> {
    let user_id = claims.user_id();

    info!(
        user_id = %user_id,
        file_size = content_length.0.0,
        mime_type = %content_type.0,
        filename = %medium_item_opts.filename,
        "Medium upload initiated"
    );

    let data_stream = body.into_data_stream();
    let stream_reader = StreamReader::new(data_stream.map_err(std::io::Error::other));
    let boxed_reader: Box<dyn AsyncRead + Send + Unpin> = Box::new(stream_reader);

    let command = CreateMediumStreamCommand {
        user_id,
        stream: boxed_reader,
        file_size: content_length.0 .0.into(),
        mime_type: content_type.0.into(),
        filename: medium_item_opts.filename,
        medium_type: medium_opts.medium_type.map(Into::into),
        priority: Some(medium_item_opts.priority),
        date_taken: medium_item_opts.date_taken,
        camera_make: medium_item_opts.camera_make,
        camera_model: medium_item_opts.camera_model,
    };

    match state
        .medium_handlers
        .create_medium_stream
        .handle(command)
        .await
    {
        Ok(medium_id) => {
            info!(
                user_id = %user_id,
                medium_id = %medium_id,
                "Medium created successfully"
            );
            Ok((StatusCode::CREATED, Json(medium_id)))
        }
        Err(e) => {
            error!(
                user_id = %user_id,
                error = %format_error_with_backtrace(&e),
                "Failed to create medium"
            );
            Err(e)
        }
    }
}
