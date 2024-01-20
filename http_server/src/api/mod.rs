use std::sync::Arc;

use axum::Router;
use axum::routing::get;

mod medium;

pub fn app() -> Router<Arc<fotonic::Service>> {
    Router::new()
        .route("/medium", get(medium::find_all).post(medium::create_medium))
}
