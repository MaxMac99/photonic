use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use axum::response::Result;

use crate::api::medium::model::find_all::FindAllMediumInput;

pub async fn find_all(
    State(service): State<Arc<fotonic::Service>>,
    opts: Query<FindAllMediumInput>,
) -> Result<Json<String>> {
    Ok(Json(String::new()))
}