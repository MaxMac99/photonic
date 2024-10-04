use crate::{
    medium::api::{__path_create_medium, create_medium},
    state::AppState,
};
use axum::{routing::post, Router};
use common::user::User;
use jwt_authorizer::{Authorizer, IntoLayer};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(create_medium))]
pub(crate) struct MediumApi;

pub fn create_api(auth: Authorizer<User>) -> Router<AppState> {
    Router::new()
        .route("/", post(create_medium))
        .layer(auth.into_layer())
}
