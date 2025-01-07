use crate::{
    api::create_router,
    config::GlobalConfig,
    exif::Exiftool,
    flows::setup_flows,
    state::AppState,
    util::{db::init_db, events::EventBus, server::shutdown_signal_with_external_signal},
};
use opentelemetry::{
    global, propagation::TextMapCompositePropagator, trace::TracerProvider as _, KeyValue,
};
use opentelemetry_sdk::{
    propagation::{BaggagePropagator, TraceContextPropagator},
    trace::TracerProvider,
    Resource,
};
use snafu::{ResultExt, Whatever};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::mpsc};
use tracing::log::info;
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

mod album;
mod api;
mod common;
mod config;
mod error;
mod exif;
mod flows;
mod medium;
mod state;
mod storage;
mod user;
mod util;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    dotenv::dotenv().ok();

    setup_tracing()?;

    let config = Arc::new(GlobalConfig::load().await?);
    let event_bus = EventBus::new();
    let db_pool = init_db(&config.database).await?;
    let exiftool = Exiftool::new()
        .await
        .whatever_context("Could not start exiftool")?;
    let (died_tx, died_rx) = mpsc::channel(1);

    let state = AppState::new(config.clone(), event_bus, db_pool, exiftool, died_tx);

    let app = create_router(state.clone()).await?;

    let listener = TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await
        .whatever_context("Could not bind to address")?;

    setup_flows(state.clone());

    info!("Starting importer API");
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal_with_external_signal(died_rx))
        .await
        .whatever_context("Could not start server")?;

    Ok(())
}

fn setup_tracing() -> Result<(), Whatever> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .whatever_context("Could no create span exporter")?;
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "photonic",
        )]))
        .build();

    let propagators = TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]);
    global::set_text_map_propagator(propagators);

    let telemetry_layer = tracing_opentelemetry::layer()
        .with_error_records_to_exceptions(true)
        .with_tracer(provider.tracer("photonic"));
    global::set_tracer_provider(provider);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .with(telemetry_layer)
        .init();
    Ok(())
}
