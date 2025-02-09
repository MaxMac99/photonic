use crate::{
    error::Result,
    state::{AppState, ArcConnection},
    user::{
        service::{create_or_update_user, get_user},
        UserInput, UserStats,
    },
};
use axum::{debug_handler, extract::State, Json};
use jwt_authorizer::{layer::AuthorizationLayer, JwtClaims};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router(state: AppState, authorization: AuthorizationLayer<UserInput>) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(user_stats))
        .layer(authorization)
        .with_state(state)
}

#[tracing::instrument(skip(state))]
#[debug_handler]
#[utoipa::path(
    get,
    path = "/stats",
    tag = "user",
    responses(
        (status = 200, content_type = "application/json", description = "Statistics of the current user", body = UserStats),
    ),
)]
async fn user_stats(
    State(state): State<AppState>,
    JwtClaims(user): JwtClaims<UserInput>,
) -> Result<Json<UserStats>> {
    let mut transaction = state.begin_transaction().await?;
    let arc_conn = ArcConnection::new(&mut *transaction);
    create_or_update_user(arc_conn.clone(), user.clone().into()).await?;

    let user_details = get_user(arc_conn.clone(), user.sub).await?;
    transaction.commit().await?;

    let stats = UserStats {
        quota: user.quota.clone(),
        quota_used: user_details.quota_used.clone(),
        albums: 0,
        media: 0,
    };

    Ok(Json(stats))
}
