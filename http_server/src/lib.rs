use std::sync::Arc;

use axum::Router;
use jwt_authorizer::{JwtAuthorizer, Validation};
use snafu::{ResultExt, Whatever};
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing::log::{debug, info};

use photonic::{config::Config, service::Service};

use crate::{api::user::User, config::ServerConfig};

mod api;
mod config;
mod error;
mod error;

pub async fn run() -> Result<(), Whatever> {
    let config = Arc::new(Config::load().await?);
    let server_config = Arc::new(ServerConfig::load()?);
    let service = Arc::new(Service::new(config.clone()).await?);

    let address = "0.0.0.0:8080";
    let listener = TcpListener::bind(address)
        .await
        .whatever_context("Could not bind to address")?;
    info!(
        "Starting photonic server at {}",
        listener.local_addr().unwrap()
    );

    let app = Router::new()
        .nest("/api/v1", api::app(auth))
        .with_state(AppState {
            service,
            server_config,
        })
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .whatever_context("Serve failed")?;

    Ok(())
}
