use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Result,
    Json,
};

use crate::api::medium::model::find_all::FindAllMediumInput;

pub async fn find_all(
    State(service): State<Arc<fotonic::Service>>,
    opts: Query<FindAllMediumInput>,
) -> Result<Json<String>> {
    Ok(Json(String::new()))
}
