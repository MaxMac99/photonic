use crate::{medium_item::model::CreateMediumItemInput, state::AppState};
use byte_unit::Byte;
use chrono::Utc;
use common::{
    error::{QuotaExceededSnafu, Result},
    medium_item::MediumItemType,
    stream::events::{MediumItemCreatedEvent, StorageLocation},
    user::{get_current_quota_usage, User},
};
use mime_serde_shim::Wrapper as Mime;
use std::future::Future;
use tokio::fs;
use uuid::Uuid;

pub async fn create_medium_item<F, Fut>(
    state: AppState,
    tmp_file: F,
    filesize: Byte,
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
    let usage = get_current_quota_usage(state.ksql_db, user_id).await?;
    if usage.as_u64() + filesize.as_u64() > user.quota.as_u64() {
        return QuotaExceededSnafu.fail();
    }

    let tmp_path = tmp_file().await?;
    let metadata = fs::metadata(tmp_path.full_path(&state.config.storage)).await?;
    let id = Uuid::new_v4();

    let event = MediumItemCreatedEvent {
        id,
        medium_id,
        medium_item_type,
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
