pub use core::Error;

pub async fn run() -> Result<(), Error> {
    http_server::run().await
}