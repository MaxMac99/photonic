use std::sync::Arc;

use axum::{routing::get, Router};

mod medium;

pub fn app() -> Router<Arc<fotonic::Service>> {
    Router::new()
        .route("/media", get(medium::find_all).post(medium::create_medium))
        .route(
            "/media/:medium_id/originals/:item_id/raw",
            get(medium::get_medium_original_raw),
        )
        .route(
            "/media/:medium_id/edits/:item_id/raw",
            get(medium::get_medium_edit_raw),
        )
}
