use crate::{
    server::{auth::create_auth, info, AppState},
    Config,
};
use axum::{
    http::StatusCode,
    routing::{get, Route},
    Router,
};
use jwt_authorizer::IntoLayer;
use snafu::Whatever;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub async fn create_routes(app_state: AppState) -> Result<Router, Whatever> {
    let auth = create_auth(&app_state.config).await?;
    Ok(Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .layer(auth.into_layer())
                .route("/ping", get(ping))
                .route("/info", get(info::info)),
        )
        .with_state(app_state)
        .layer(TraceLayer::new_for_http()))
}

pub async fn ping() -> StatusCode {
    StatusCode::NO_CONTENT
}
