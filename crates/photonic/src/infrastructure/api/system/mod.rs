pub mod dto;
mod info;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::infrastructure::api::state::AppState;

/// Returns routes with OpenAPI metadata. No state needed.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(info::system_info))
}

/// Full router with state.
pub fn router(state: AppState) -> OpenApiRouter {
    routes().with_state(state)
}
