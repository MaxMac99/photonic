use axum::middleware;
use jwt_authorizer::layer::AuthorizationLayer;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::infrastructure::{
    api::state::AppState,
    auth::{ensure_user_exists, JwtUserClaims},
};

pub mod dto;
mod get_tasks;

/// Returns routes with OpenAPI metadata. No state or layers needed.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        // route /
        .routes(routes!(
            get_tasks::get_tasks,
        ))
}

/// Full router with authorization layers and state.
pub fn router(state: AppState, authorization: AuthorizationLayer<JwtUserClaims>) -> OpenApiRouter {
    routes()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            ensure_user_exists,
        ))
        .layer(authorization)
        .with_state(state)
}
