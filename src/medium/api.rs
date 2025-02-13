use crate::{
    api::Binary,
    error::Result,
    medium::{
        model::CreateMediumInput, service, CreateMediumItemInput, FindAllMediaOptions,
        GetMediumPreviewOptions, MediumItemType, MediumResponse,
    },
    state::{AppState, ArcConnection},
    storage::service::store_tmp_from_stream,
    user::{service::create_or_update_user, UserInput},
};
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
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
        .routes(routes!(create_medium, get_all_media, delete_medium,))
        .routes(routes!(add_medium_item, get_medium_item))
        .routes(routes!(get_medium_preview))
        .layer(authorization)
        .with_state(state)
}

#[tracing::instrument(skip(state))]
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
async fn create_medium(
    State(state): State<AppState>,
    content_length: TypedHeader<ContentLength>,
    content_type: TypedHeader<ContentType>,
    Query(medium_opts): Query<CreateMediumInput>,
    Query(medium_item_opts): Query<CreateMediumItemInput>,
    JwtClaims(user): JwtClaims<UserInput>,
    body: Body,
) -> Result<(StatusCode, Json<Uuid>)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    let extension = medium_item_opts.extension.clone();
    let size = Byte::from_u64(content_length.0 .0);
    let medium_item_event = service::create_medium(
        state.clone(),
        arc_conn,
        |conn, medium_item_id| {
            store_tmp_from_stream(
                state.clone(),
                conn,
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

#[tracing::instrument(skip(state))]
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
async fn get_all_media(
    State(state): State<AppState>,
    Query(opts): Query<FindAllMediaOptions>,
    JwtClaims(user): JwtClaims<UserInput>,
) -> Result<(StatusCode, Json<Vec<MediumResponse>>)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    let response = service::get_media(arc_conn, user, opts).await?;
    transaction.commit().await?;
    Ok((StatusCode::OK, Json::from(response)))
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    delete,
    path = "/{medium_id}",
    tag = "medium",
    responses(
        (status = 204, content_type = "application/json", description = "Deletes the medium"),
    ),
    params(
        ("medium_id" = Uuid, Path, description = "The id of the medium to delete"),
    ),
)]
async fn delete_medium(
    Path(medium_id): Path<Uuid>,
    State(state): State<AppState>,
    JwtClaims(user): JwtClaims<UserInput>,
) -> Result<StatusCode> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    service::delete_medium(arc_conn, user, medium_id).await?;

    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
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
    params(CreateMediumItemInput),
)]
async fn add_medium_item(
    State(state): State<AppState>,
    Path((medium_id, format)): Path<(Uuid, MediumItemType)>,
    content_length: TypedHeader<ContentLength>,
    content_type: TypedHeader<ContentType>,
    Query(medium_opts): Query<CreateMediumInput>,
    Query(medium_item_opts): Query<CreateMediumItemInput>,
    JwtClaims(user): JwtClaims<UserInput>,
    body: Body,
) -> Result<(StatusCode, Json<Uuid>)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    let extension = medium_item_opts.extension.clone();
    let size = Byte::from_u64(content_length.0 .0);
    let medium_item_event = service::add_medium_item(
        state.clone(),
        arc_conn,
        |inner_transaction, medium_item_id| {
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
        medium_id,
        format,
        medium_item_opts,
        content_type.0.into(),
    )
    .await?;
    transaction.commit().await?;

    state.event_bus.publish(medium_item_event.clone()).await?;

    Ok((StatusCode::CREATED, Json::from(medium_item_event.medium_id)))
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}/item/{item_id}/raw",
    tag = "medium",
    responses(
        (status = 200, description = "The raw file", body = Binary, content_type = "*/*", headers(
            ("content-type" = String)
        )),
    ),
)]
async fn get_medium_item(
    State(state): State<AppState>,
    Path((medium_id, item_id)): Path<(Uuid, Uuid)>,
    JwtClaims(user): JwtClaims<UserInput>,
) -> Result<(StatusCode, HeaderMap, Body)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;
    let (mime, stream) =
        service::get_raw_medium(state.clone(), arc_conn, user, medium_id, item_id).await?;
    transaction.commit().await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime.to_string().parse().unwrap());
    Ok((StatusCode::OK, headers, Body::from_stream(stream)))
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/{medium_id}/preview",
    tag = "medium",
    responses(
        (status = 200, description = "The raw file", body = Binary, content_type = "*/*", headers(
            ("content-type" = String)
        )),
    ),
    params(GetMediumPreviewOptions),
)]
async fn get_medium_preview(
    State(state): State<AppState>,
    Path(medium_id): Path<Uuid>,
    Query(opts): Query<GetMediumPreviewOptions>,
    JwtClaims(user): JwtClaims<UserInput>,
) -> Result<(StatusCode, HeaderMap, Body)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;
    let (mime, stream) =
        service::get_medium_preview(state.clone(), arc_conn, user, medium_id, opts).await?;

    transaction.commit().await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime.to_string().parse().unwrap());
    Ok((StatusCode::OK, headers, Body::from_stream(stream)))
}
