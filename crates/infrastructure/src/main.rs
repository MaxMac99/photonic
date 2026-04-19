use infrastructure::{config::GlobalConfig, run_server, setup_tracing};
use snafu::Whatever;

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    dotenv::dotenv().ok();

    setup_tracing()?;

    let config = GlobalConfig::load().await?;

    // Run server with configured port (blocks until shutdown)
    let handle = run_server(config, None).await?;

    // Keep main thread alive (server runs in background task)
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");

    handle.shutdown().await;

    Ok(())
}
