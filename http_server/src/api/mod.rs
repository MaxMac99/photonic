use axum::{
    routing::{delete, get, post},
    Router,
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
            "/media/:medium_id/:format/:item_id/raw",
            get(media::get_medium_item_raw),
        )
        .route("/media/:medium_id/:format/raw", post(media::add_raw))
        .layer(auth.into_layer())
        .route("/ping", get(ping::ping))
        .route("/info", get(info::info))
}
