use std::sync::Arc;

use axum::{routing::get, Router};

mod medium;

pub fn app() -> Router<Arc<fotonic::Service>> {
    Router::new()
        .route("/medium", get(medium::find_all).post(medium::create_medium))
}
