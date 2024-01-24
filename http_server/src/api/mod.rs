use std::sync::Arc;

use axum::{routing::get, Router};

mod media;

pub fn app() -> Router<Arc<fotonic::Service>> {
    Router::new()
        .route("/media", get(media::find_all).post(media::create_medium))
        .route(
            "/media/:medium_id/originals/:item_id/raw",
            get(media::get_medium_original_raw),
        )
        .route(
            "/media/:medium_id/edits/:item_id/raw",
            get(media::get_medium_edit_raw),
        )
        .route(
            "/media/:medium_id/preview/raw",
            get(media::get_medium_preview_raw),
        )
}
