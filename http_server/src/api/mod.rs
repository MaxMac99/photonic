use axum::{
    Router,
    routing::{delete, get},
};
use jwt_authorizer::{Authorizer, IntoLayer};

use crate::{api::user::User, AppState};

mod info;
mod media;
mod ping;
pub(crate) mod user;

pub fn app(auth: Authorizer<User>) -> Router<AppState> {
    Router::new()
        .route("/media", get(media::find_all).post(media::create_medium))
        .route("/media/:medium_id", delete(media::delete_medium))
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
        .route(
            "/media/:medium_id/sidecars/:item_id/raw",
            get(media::get_medium_sidecar_raw),
        )
        .layer(auth.into_layer())
        .route("/ping", get(ping::ping))
        .route("/info", get(info::info))
}
