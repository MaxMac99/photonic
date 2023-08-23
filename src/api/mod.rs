mod create_medium;

pub fn register_urls(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create_medium::create_medium);
}
