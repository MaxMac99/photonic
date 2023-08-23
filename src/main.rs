use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use env_logger::Env;

use repository::MongoRepo;
use repository::PhotoRepo;

mod api;
mod models;
mod repository;
mod common;
mod core;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .init();

    rexiv2::initialize().unwrap();
    let store = PhotoRepo::init().await;
    let store_data = Data::new(store);

    let db = MongoRepo::init().await;
    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(db_data.clone())
            .app_data(store_data.clone())
            .configure(api::register_urls)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
