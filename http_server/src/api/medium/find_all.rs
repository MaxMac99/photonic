use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, Responder, Result, web};
use actix_web::web::Query;

use crate::api::medium::model::input::FindAllMediumInput;

pub async fn find_all(
    ctx: web::Data<Arc<core::Service>>,
    req: HttpRequest,
    opts: Query<FindAllMediumInput>,
) -> Result<impl Responder> {
    Ok(HttpResponse::Ok())
}