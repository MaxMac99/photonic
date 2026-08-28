use axum::middleware;
use jwt_authorizer::layer::AuthorizationLayer;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    api::state::AppState,
    auth::{ensure_user_exists, JwtUserClaims},
};

mod add_medium_item;
mod create_medium;
mod delete_medium;
pub mod dto;
mod get_all_media;
mod get_medium;
mod get_medium_item;
mod get_medium_metadata;
mod get_medium_preview;

/// Returns routes with OpenAPI metadata. No state or layers needed.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        // route /
        .routes(routes!(
            create_medium::create_medium,
            get_all_media::get_all_media,
        ))
        // route /{medium_id}
        .routes(routes!(
            get_medium::get_medium,
            delete_medium::delete_medium,
        ))
        // route /{medium_id}/metadata
        .routes(routes!(get_medium_metadata::get_medium_metadata))
        // route /{medium_id}/preview
        .routes(routes!(get_medium_preview::get_medium_preview))
        // route /{medium_id}/item/{format}
        .routes(routes!(add_medium_item::add_medium_item,))
        // route /{medium_id}/item/{item_id}/raw
        .routes(routes!(get_medium_item::get_medium_item,))
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
