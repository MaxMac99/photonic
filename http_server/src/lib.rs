use std::sync::Arc;

use axum::Router;
use snafu::{ResultExt, Whatever};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::log::{debug, info};

pub use error::ResponseError;
use fotonic::config::Config;
use fotonic::service::Service;

mod api;
mod error;

pub async fn run() -> Result<(), Whatever> {
    let config = Arc::new(Config::load().await?);
    let service = Arc::new(Service::new(config.clone()).await?);

    let address = "0.0.0.0:8080";
    let listener = TcpListener::bind(address).await
        .whatever_context("Could not bind to address")?;
    info!("Starting fotonic server at {}", listener.local_addr().unwrap());

    let app = Router::new()
        .nest("/api/v1", api::app())
        .with_state(service)
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .whatever_context("Serve failed")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    debug!("signal received, starting graceful shutdown");
}
