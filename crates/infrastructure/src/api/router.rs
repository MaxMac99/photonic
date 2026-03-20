use axum::routing::Router;
use jwt_authorizer::IntoLayer;
use snafu::Whatever;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use super::{medium, system};
use crate::{api::state::AppState, server::setup_auth};

#[derive(utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct Binary(String);

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:8080", description = "Development server"),
        (url = "https://infrastructure.mvissing.de", description = "Staging server"),
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

pub async fn create_router(state: AppState) -> Result<Router, Whatever> {
    let (router, api) = create_router_with_api(state).await?;
    let router = router.merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", api));
    Ok(router)
}

pub async fn create_router_with_api(
    state: AppState,
) -> Result<(Router, utoipa::openapi::OpenApi), Whatever> {
    let auth = setup_auth(&state.config.clone().server).await?.into_layer();
    Ok(OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest(
            "/api/v1/medium",
            medium::router(state.clone(), auth.clone()),
        )
        // .nest(
        //     "/api/v1/album",
        //     album::api::router(state.clone(), auth.clone()),
        // )
        // .nest(
        //     "/api/v1/user",
        //     user::api::router(state.clone(), auth.clone()),
        // )
        .nest("/api/v1/system", system::router(state.clone()))
        .layer(TraceLayer::new_for_http())
        .split_for_parts())
}

pub fn create_api() -> utoipa::openapi::OpenApi {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest(
            "/api/v1/medium",
            medium::routes()
        )
        // .nest(
        //     "/api/v1/album",
        //     album::api::router(),
        // )
        // .nest(
        //     "/api/v1/user",
        //     user::api::router(),
        // )
        .nest("/api/v1/system", system::routes())
        .into_openapi()
}
