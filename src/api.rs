use crate::{album, medium, state::AppState, util::server::setup_auth};
use axum::Router;
use jwt_authorizer::IntoLayer;
use snafu::Whatever;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "medium", description = "Medium API"),
        (name = "album", description = "Album API"),
    ),
)]
pub struct ApiDoc;

pub async fn create_router<'e>(state: AppState) -> Result<Router, Whatever> {
    let auth = setup_auth(&state.config.clone().server).await?.into_layer();
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest(
            "/api/v1/medium",
            medium::api::router(state.clone(), auth.clone()),
        )
        .nest(
            "/api/v1/album",
            album::api::router(state.clone(), auth.clone()),
        )
        .layer(TraceLayer::new_for_http())
        .split_for_parts();
    let router = router.merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", api));
    Ok(router)
}
