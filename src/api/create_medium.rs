use actix_web::{error::ContentTypeError, HttpMessage, HttpRequest, HttpResponse, post, Responder, Result, web::{self, Data}};
use actix_web::web::Query;

use crate::{repository::MongoRepo, repository::PhotoRepo};
use crate::core::medium;
use crate::core::medium::CreateMediumOpts;

#[post("/media")]
pub async fn create_medium(
    db: Data<MongoRepo>,
    store: Data<PhotoRepo>,
    req: HttpRequest,
    opts: Query<CreateMediumOpts>,
    body: web::Bytes,
) -> Result<impl Responder> {
    let mime = req.mime_type()?.ok_or(ContentTypeError::ParseError)?;

    let id = medium::create_medium(&db, &store, &opts, mime, &body).await?;

    Ok(HttpResponse::Ok()
        .body(id.to_hex()))
}
