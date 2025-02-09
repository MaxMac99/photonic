use crate::{album, medium, state::AppState, system, user, util::server::setup_auth};
use axum::Router;
use jwt_authorizer::IntoLayer;
use snafu::Whatever;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct Binary(String);

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:8080", description = "Development server"),
        (url = "https://photonic.mvissing.de", description = "Staging server"),
    ),
    tags(
        (name = "medium", description = "Medium API"),
        (name = "album", description = "Album API"),
        (name = "system", description = "System API"),
        (name = "user", description = "User API"),
    ),
    components(
        schemas(
            Binary,
        ),
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
        .nest(
            "/api/v1/user",
            user::api::router(state.clone(), auth.clone()),
        )
        .nest("/api/v1/info", system::api::router(state.clone()))
        .layer(TraceLayer::new_for_http())
        .split_for_parts();
    let router = router.merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", api));
    Ok(router)
}
