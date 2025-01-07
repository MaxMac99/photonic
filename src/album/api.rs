use crate::{
    album::{
        model::{AlbumResponse, CreateAlbumInput, FindAllAlbumsOptions},
        service,
    },
    error::Result,
    state::{AppState, ArcConnection},
    user::{service::create_or_update_user, UserInput},
};
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use jwt_authorizer::{layer::AuthorizationLayer, JwtClaims};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

pub fn router(state: AppState, authorization: AuthorizationLayer<UserInput>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(create_album, find_all_albums))
        .layer(authorization)
        .with_state(state)
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    post,
    path = "",
    tag = "album",
    request_body(
        content = CreateAlbumInput,
        content_type = "application/json"
    ),
    responses(
        (status = 201, content_type = "application/json", description = "The id of the newly created album", body = Uuid),
    ),
)]
async fn create_album(
    State(state): State<AppState>,
    JwtClaims(user): JwtClaims<UserInput>,
    Json(opts): Json<CreateAlbumInput>,
) -> Result<(StatusCode, Json<Uuid>)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    let album_id = service::create_album(arc_conn, user, opts).await?;

    Ok((StatusCode::CREATED, Json(album_id)))
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "",
    tag = "album",
    responses(
        (status = 200, content_type = "application/json", description = "Information on all albums", body = Vec<AlbumResponse>),
    ),
    params(FindAllAlbumsOptions),
)]
async fn find_all_albums(
    State(state): State<AppState>,
    JwtClaims(user): JwtClaims<UserInput>,
    Query(opts): Query<FindAllAlbumsOptions>,
) -> Result<(StatusCode, Json<Vec<AlbumResponse>>)> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);

    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    let album_id = service::find_albums(arc_conn, user, opts).await?;

    Ok((StatusCode::CREATED, Json(album_id)))
}
