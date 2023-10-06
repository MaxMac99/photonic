use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

mod api;

pub async fn run() -> Result<(), core::Error> {
    let config = Arc::new(core::Config::load().await?);
    let service = Arc::new(core::Service::new(config.clone()).await?);

    let endpoint = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Starting fotonic server. endpoint={:?}", &endpoint);

    let app = Router::new()
        .nest("/api/v1", api::app())
        .with_state(service)
        .layer(TraceLayer::new_for_http());

    hyper::Server::bind(&endpoint)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| core::Error::Internal(err.to_string()))?;

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
