use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use log::info;

mod api;

pub async fn run() -> Result<(), core::Error> {
    let config = Arc::new(core::Config::load().await?);
    let service = Arc::new(core::Service::new(config.clone()).await);

    let endpoint = ("0.0.0.0", 8080);
    info!("Starting fotonic server. endpoint={:?}", &endpoint);
    HttpServer::new(move || {
        App::new()
            .app_data(Arc::clone(&service))
            .service(web::scope("/api/v1")
                .service(web::scope("/medium")
                    .service(web::resource("")
                        .route(web::post().to(api::medium::create_medium)))))
    })
        .bind(endpoint)?
        .run()
        .await?;

    Ok(())
}
