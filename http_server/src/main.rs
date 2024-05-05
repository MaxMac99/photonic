use snafu::Whatever;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    http_server::run().await
}
