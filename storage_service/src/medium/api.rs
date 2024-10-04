use crate::{
    medium::{model::CreateMediumInput, service},
    state::AppState,
    storage::stream::store_tmp_from_stream,
};
use axum::{
    body::Body,
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::{headers::ContentType, TypedHeader};
use common::{error::Result, user::User};
use jwt_authorizer::JwtClaims;
use uuid::Uuid;

#[debug_handler]
#[utoipa::path(
    post,
    path = "/",
    request_body(
        content = Vec<u8>,
        content_type = "*"),
    responses(
        (status = 201, content_type = "application/json", description = "Uploaded a file", body = Uuid),
    ),
    params(CreateMediumInput),
)]
pub async fn create_medium(
    State(state): State<AppState>,
    content_type: TypedHeader<ContentType>,
    Query(opts): Query<CreateMediumInput>,
    JwtClaims(user): JwtClaims<User>,
    body: Body,
) -> Result<(StatusCode, Json<Uuid>)> {
    let extension = opts.extension.clone();
    let response = service::create_medium(
        state.clone(),
        || async move { store_tmp_from_stream(state, body.into_data_stream(), extension).await },
        user,
        opts,
        content_type.0.into(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json::from(response)))
}
