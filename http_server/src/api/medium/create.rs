use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, Result, web};
use actix_web::web::Query;

use core::Error;

use crate::api::medium::model::input::CreateMediumInput;

pub async fn create_medium(
    ctx: web::Data<Arc<core::Service>>,
    req: HttpRequest,
    opts: Query<CreateMediumInput>,
    body: web::Bytes,
) -> Result<impl Responder> {
    let mime = req.mime_type()?.ok_or(Error::InvalidArgument(String::from("Could not find mime type")))?;

    let input = opts.into_inner();
    let create_medium = core::CreateMediumInput {
        album_id: input.album_id,
        filename: input.filename,
        tags: input.tags,
        date_taken: input.date_taken,
        mime,
    };
    let id = ctx.create_medium(create_medium, body.as_ref()).await?;

    Ok(HttpResponse::Ok()
        .body(id.to_hex()))
}
