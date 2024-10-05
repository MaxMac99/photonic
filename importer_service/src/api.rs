use crate::{config::ImporterWorkerConfig, medium, state::AppState};
use axum::Router;
use common::{medium::MediumType, server::setup_auth};
use snafu::Whatever;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/api/v1/medium", api = medium::router::MediumApi),
    ),
    tags(
        (name = "medium", description = "Medium API")
    ),
    components(
        schemas(MediumType),
    ),
)]
pub struct ApiDoc;

pub async fn create_app(
    config: &Arc<ImporterWorkerConfig>,
    state: AppState,
) -> Result<Router, Whatever> {
    let auth = setup_auth(&config.clone().server).await?;
    let app = Router::new()
        .nest("/api/v1/medium", medium::router::create_api(auth))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()));
    Ok(app)
}
