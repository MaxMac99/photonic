pub use core::Error;

pub async fn run() -> Result<(), Error> {
    super::init_logger();
    dotenv::dotenv()
        .map_err(|err| Error::Internal(format!("Could not read dot env: {}", err.to_string())))?;

    http_server::run().await
}