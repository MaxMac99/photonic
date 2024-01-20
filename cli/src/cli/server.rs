use snafu::Whatever;

pub async fn run() -> Result<(), Whatever> {
    http_server::run().await
}