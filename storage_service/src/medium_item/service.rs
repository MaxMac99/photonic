use crate::{
    medium_item::{
        model::{CreateMediumItemInput, MediumItem, MediumItemType},
        repo::{add_medium_item, find_medium_item_by_id},
    },
    state::AppState,
    user::service::create_or_update_user,
};
use byte_unit::Byte;
use chrono::Utc;
use common::{
    error::Result,
    stream::events::{MediumItemCreatedEvent, MediumItemExifLoadedEvent, StorageLocation},
    user::User,
};
use mime_serde_shim::Wrapper as Mime;
use std::future::Future;
use tokio::fs;
use tracing::log::debug;
use uuid::Uuid;

pub async fn create_medium_item<F, Fut>(
    state: AppState,
    tmp_file: F,
    user: User,
    opts: CreateMediumItemInput,
    mime: Mime,
    medium_id: Uuid,
    medium_item_type: MediumItemType,
) -> Result<Uuid>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<StorageLocation>>,
{
    let user_id = user.sub;
    create_or_update_user(&state, user.into()).await?;
    let tmp_path = tmp_file().await?;
    let metadata = fs::metadata(tmp_path.full_path(&state.config.storage)).await?;

    let item = MediumItem {
        id: Default::default(),
        medium_id,
        medium_item_type,
        mime: mime.clone(),
        filename: opts.filename.clone(),
        location: tmp_path.clone(),
        filesize: Byte::from_u64(metadata.len()),
        priority: opts.priority,
        taken_at: opts.date_taken,
        last_saved: Default::default(),
        deleted_at: None,
        width: None,
        height: None,
    };
    let id = add_medium_item(&state.db_pool, item).await?;

    let event = MediumItemCreatedEvent {
        id,
        medium_id,
        location: tmp_path,
        size: Byte::from_u64(metadata.len()),
        mime,
        filename: opts.filename,
        extension: opts.extension,
        user: user_id,
        priority: opts.priority,
        date_taken: opts.date_taken,
        date_added: Utc::now(),
    };
    state.producer.produce(event).await?;

    Ok(id)
}

pub async fn handle_exif_loaded(state: AppState, event: MediumItemExifLoadedEvent) -> Result<()> {
    let mut item = find_medium_item_by_id(&state.db_pool, event.id).await?;
    debug!("Found item: {:?}", item);
    Ok(())
}
