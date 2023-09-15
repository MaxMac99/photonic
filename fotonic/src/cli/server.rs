pub async fn run() -> Result<(), core::Error> {
    super::init_logger();

    http_server::run().await
}