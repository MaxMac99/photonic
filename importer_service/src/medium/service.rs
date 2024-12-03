use crate::{
    medium::model::CreateMediumInput, medium_item::service::create_medium_item, state::AppState,
};
use byte_unit::Byte;
use common::{
    error::Result, medium_item::MediumItemType,
    stream::events::StorageLocation, user::User,
};
use mime::Mime;
use mime_serde_shim::Wrapper;
use std::future::Future;
use uuid::Uuid;
use common::medium::MediumType;

pub async fn create_medium<F, Fut>(
    state: AppState,
    tmp_file: F,
    filesize: Byte,
    user: User,
    opts: CreateMediumInput,
    mime: Mime,
) -> Result<Uuid>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<StorageLocation>>,
{
    let medium_type = opts
        .medium_type
        .unwrap_or_else(|| MediumType::from(mime.clone()));
    let id = Uuid::new_v4();

    let event = MediumUpdatedEvent
    state.producer.produce()

    create_medium_item(
        state,
        tmp_file,
        filesize,
        user,
        opts.into(),
        Wrapper(mime),
        id,
        MediumItemType::Original,
    )
    .await?;

    Ok(id)
}
