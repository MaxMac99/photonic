use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use log::info;

mod api;

#[derive(Clone)]
struct ServerContext {}

pub async fn run() -> Result<(), core::Error> {
    let context = Arc::new(ServerContext {});

    let endpoint = ("0.0.0.0", 8080);
    info!("Starting fotonic server. endpoint={:?}", &endpoint);
    HttpServer::new(move || {
        App::new()
            .app_data(Arc::clone(&context))
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
