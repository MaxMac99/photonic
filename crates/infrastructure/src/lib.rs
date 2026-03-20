pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod di;
pub mod events;
pub mod external;
pub mod persistence;
pub mod serde;
pub mod server;
mod shared;
pub mod storage;

use std::{net::SocketAddr, sync::Arc};

use opentelemetry::{
    global, propagation::TextMapCompositePropagator, trace::TracerProvider as _, KeyValue,
};
use opentelemetry_sdk::{
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::TracerProvider,
    Resource,
};
use snafu::{ResultExt, Whatever};
use tokio::{net::TcpListener, sync::mpsc};
use tracing::log::info;
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

use crate::{
    api::{router::create_router, state::AppState},
    config::GlobalConfig,
    db::init_db,
    di::Container,
    server::shutdown_signal_with_external_signal,
};

/// Server handle that can be used to control a running server
pub struct ServerHandle {
    pub addr: SocketAddr,
    pub shutdown_tx: mpsc::Sender<bool>,
}

impl ServerHandle {
    /// Shutdown the server gracefully
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true).await;
    }
}

/// Set up tracing and telemetry for production
pub fn setup_tracing() -> Result<(), Whatever> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .whatever_context("Could no create span exporter")?;
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "infrastructure",
        )]))
        .build();

    let propagators = TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]);
    global::set_text_map_propagator(propagators);

    let telemetry_layer = tracing_opentelemetry::layer()
        .with_error_records_to_exceptions(true)
        .with_tracer(provider.tracer("infrastructure"));
    global::set_tracer_provider(provider);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_thread_names(true).with_thread_ids(true))
        .with(telemetry_layer)
        .init();
    Ok(())
}

/// Set up simple logging for tests (no OpenTelemetry)
pub fn setup_test_tracing() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Enable backtraces for snafu errors
        std::env::set_var("RUST_BACKTRACE", "1");

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            // Human-readable format for tests
            .pretty()
            // Show file and line numbers for easier debugging
            .with_file(true)
            .with_line_number(true)
            // Show which code span we're in
            .with_target(true)
            // Include thread names and IDs for async debugging
            .with_thread_names(true)
            .with_thread_ids(true)
            // Use test writer to capture output
            .with_test_writer()
            .init();
    });
}

/// Run the application server
///
/// If port is None, uses the port from config
/// If port is Some(0), a random available port will be assigned
/// Returns the server handle with the actual bound address
pub async fn run_server(
    config: Arc<GlobalConfig>,
    port: Option<u16>,
) -> Result<ServerHandle, Whatever> {
    let db_pool = init_db(&config.database).await?;
    let (died_tx, died_rx) = mpsc::channel(1);

    let container = Container::new(config.clone(), db_pool).await?;
    let state = AppState::new(container).await?;
    let app = create_router(state.clone()).await?;

    let port = port.unwrap_or(config.server.port);
    let listener = TcpListener::bind(format!("{}:{}", config.server.host, port))
        .await
        .whatever_context("Could not bind to address")?;

    let addr = listener
        .local_addr()
        .whatever_context("Could not get local address")?;

    info!("Starting server on {}", addr);

    // Spawn server in background
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal_with_external_signal(died_rx))
            .await
            .expect("Server error");
    });

    Ok(ServerHandle {
        addr,
        shutdown_tx: died_tx,
    })
}
