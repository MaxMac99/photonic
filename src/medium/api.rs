use crate::{
    error::Result,
    medium::{
        model::CreateMediumInput, service, CreateMediumItemInput, FindAllMediaOptions,
        MediumResponse,
    },
    state::{AppState, Transaction},
    storage::service::store_tmp_from_stream,
    user::{service::create_or_update_user, UserInput},
};
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
use byte_unit::Byte;
use futures_util::FutureExt;
use jwt_authorizer::{layer::AuthorizationLayer, JwtClaims};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

pub fn router(state: AppState, authorization: AuthorizationLayer<UserInput>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_medium, find_all_media))
        .layer(authorization)
        .with_state(state)
}

#[debug_handler]
#[utoipa::path(
    post,
    path = "",
    tag = "medium",
    request_body(
        content = Vec<u8>,
        content_type = "*"
    ),
    responses(
        (status = 201, content_type = "application/json", description = "Uploaded a file", body = Uuid),
    ),
    params(CreateMediumInput, CreateMediumItemInput),
)]
async fn create_medium(
    State(state): State<AppState>,
    content_length: TypedHeader<ContentLength>,
    content_type: TypedHeader<ContentType>,
    Query(medium_opts): Query<CreateMediumInput>,
    Query(medium_item_opts): Query<CreateMediumItemInput>,
    JwtClaims(user): JwtClaims<UserInput>,
    body: Body,
) -> Result<(StatusCode, Json<Uuid>)> {
    let extension = medium_item_opts.extension.clone();
    let mut transaction = state.begin_transaction().await?;
    create_or_update_user(&mut transaction, user.clone().into()).await?;
    let size = Byte::from_u64(content_length.0 .0);
    let medium_item_event = service::create_medium(
        state.clone(),
        &mut transaction,
        |inner_transaction: &mut Transaction, medium_item_id| {
            store_tmp_from_stream(
                state.clone(),
                inner_transaction,
                medium_item_id,
                body.into_data_stream(),
                extension,
            )
            .boxed()
        },
        size,
        user,
        medium_opts,
        medium_item_opts,
        content_type.0.into(),
    )
    .await?;
    transaction.commit().await?;

    state.event_bus.publish(medium_item_event.clone()).await?;

    Ok((StatusCode::CREATED, Json::from(medium_item_event.medium_id)))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    tag = "medium",
    responses(
        (status = 200, content_type = "application/json", description = "Gets all media. Can be filtered by date", body = [MediumResponse]),
    ),
    params(FindAllMediaOptions),
)]
async fn find_all_media(
    State(state): State<AppState>,
    Query(opts): Query<FindAllMediaOptions>,
    JwtClaims(user): JwtClaims<UserInput>,
) -> Result<(StatusCode, Json<Vec<MediumResponse>>)> {
    let mut transaction = state.begin_transaction().await?;
    create_or_update_user(&mut transaction, user.clone().into()).await?;
    let response = service::find_media(&mut transaction, user, opts).await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json::from(response)))
}
