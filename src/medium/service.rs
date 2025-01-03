use crate::{
    error::{QuotaExceededSnafu, Result},
    exif::MediumItemExifLoadedEvent,
    medium::{
        model::CreateMediumInput,
        repo,
        repo::{FullMediumItemDb, MediumDb, MediumItemDb, MediumItemInfoDb},
        CreateMediumItemInput, FindAllMediaOptions, MediumItemCreatedEvent, MediumItemResponse,
        MediumItemType, MediumResponse, MediumType,
    },
    state::{AppState, Transaction},
    storage::{service::find_locations_by_medium_item_id, StorageLocation},
    user::{service::get_user, UserInput},
};
use byte_unit::Byte;
use chrono::{FixedOffset, Utc};
use futures_util::future::{try_join_all, BoxFuture};
use mime::Mime;
use mime_serde_shim::Wrapper;
use std::sync::Arc;
use tokio::{fs, sync::Mutex};
use uuid::Uuid;

pub async fn create_medium<F>(
    state: AppState,
    transaction: &mut Transaction,
    tmp_file: F,
    filesize: Byte,
    user: UserInput,
    medium_opts: CreateMediumInput,
    medium_item_opts: CreateMediumItemInput,
    mime: Mime,
) -> Result<MediumItemCreatedEvent>
where
    F: FnOnce(&mut Transaction, Uuid) -> BoxFuture<'_, Result<StorageLocation>>,
{
    let user_id = user.sub;
    let usage = get_user(transaction, user_id).await?.quota_used;
    if usage.as_u64() + filesize.as_u64() > user.quota.as_u64() {
        return QuotaExceededSnafu.fail();
    }

    let medium_item_id = Uuid::new_v4();
    let tmp_path = tmp_file(transaction, medium_item_id).await?;

    let metadata = fs::metadata(tmp_path.full_path(&state.config.storage)).await?;
    let medium_type = medium_opts
        .medium_type
        .unwrap_or_else(|| MediumType::from(mime.clone()));

    let medium = MediumDb {
        id: Uuid::new_v4(),
        owner_id: user.sub,
        medium_type,
        leading_item_id: medium_item_id,
    };
    repo::create_medium(transaction, medium.clone()).await?;
    repo::create_medium_item(
        transaction,
        MediumItemDb {
            id: medium_item_id,
            medium_id: medium.id,
            medium_item_type: MediumItemType::Original,
            mime: mime.to_string(),
            filename: medium_item_opts.filename.clone(),
            size: metadata.len() as i64,
            priority: medium_item_opts.priority.clone(),
            last_saved: Utc::now().naive_utc(),
            deleted_at: None,
        },
    )
    .await?;
    repo::create_medium_item_info(
        transaction,
        MediumItemInfoDb {
            id: medium_item_id,
            taken_at: medium_item_opts.date_taken.map(|date| date.to_utc()),
            taken_at_timezone: medium_item_opts
                .date_taken
                .map(|date| date.offset().local_minus_utc()),
            camera_make: medium_item_opts.camera_make.clone(),
            camera_model: medium_item_opts.camera_model.clone(),
            width: None,
            height: None,
        },
    )
    .await?;

    let medium_item_event = MediumItemCreatedEvent {
        id: medium_item_id,
        medium_id: medium.id,
        medium_item_type: MediumItemType::Original,
        location: tmp_path,
        size: Byte::from_u64(metadata.len()),
        mime: Wrapper(mime),
        filename: medium_item_opts.filename,
        extension: medium_item_opts.extension,
        user: user.sub,
        priority: medium_item_opts.priority,
        date_taken: medium_item_opts.date_taken,
        camera_make: medium_item_opts.camera_make,
        camera_model: medium_item_opts.camera_model,
        date_added: Utc::now(),
    };
    Ok(medium_item_event)
}

pub async fn find_media(
    transaction: &mut Transaction,
    user: UserInput,
    opts: FindAllMediaOptions,
) -> Result<Vec<MediumResponse>> {
    let owner_id = user.sub;
    let media = repo::find_media(transaction, owner_id, opts).await?;
    let transaction_ref = Arc::new(Mutex::new(transaction));
    let result = try_join_all(
        media
            .into_iter()
            .map(|medium| create_medium_response(medium, transaction_ref.clone())),
    )
    .await?;

    Ok(result)
}

pub async fn update_medium_item_from_exif(
    transaction: &mut Transaction,
    exif: MediumItemExifLoadedEvent,
) -> Result<()> {
    let medium_item = repo::find_medium_item_info(transaction, exif.id)
        .await?
        .expect("Medium item not found");
    repo::update_medium_item_info(
        transaction,
        MediumItemInfoDb {
            id: exif.id,
            taken_at: medium_item.taken_at.or(exif.date.map(|date| date.to_utc())),
            taken_at_timezone: medium_item
                .taken_at_timezone
                .or(exif.date.map(|date| date.offset().local_minus_utc())),
            camera_make: medium_item.camera_make.or(exif.camera_make),
            camera_model: medium_item.camera_model.or(exif.camera_model),
            width: medium_item.width.or(exif.width.map(|width| width as i32)),
            height: medium_item
                .height
                .or(exif.height.map(|height| height as i32)),
        },
    )
    .await?;
    Ok(())
}

async fn create_medium_response(
    medium: MediumDb,
    transaction_ref: Arc<Mutex<&mut Transaction>>,
) -> Result<MediumResponse> {
    let mut transaction = transaction_ref.lock().await;
    let items = repo::find_medium_items_by_id(*transaction, medium.id).await?;
    drop(transaction);
    let items =
        try_join_all(items.into_iter().map(|item| {
            create_medium_item_response(item, medium.clone(), transaction_ref.clone())
        }))
        .await?;
    Ok(MediumResponse {
        id: medium.id,
        medium_type: medium.medium_type,
        items,
    })
}

async fn create_medium_item_response(
    item: FullMediumItemDb,
    medium: MediumDb,
    transaction_ref: Arc<Mutex<&mut Transaction>>,
) -> Result<MediumItemResponse> {
    let mut transaction = transaction_ref.lock().await;
    let locations = find_locations_by_medium_item_id(*transaction, item.id).await?;
    Ok(MediumItemResponse {
        id: item.id,
        is_primary: item.id == medium.leading_item_id,
        medium_item_type: item.medium_item_type,
        mime: item.mime.parse().unwrap(),
        filename: item.filename,
        locations,
        filesize: Byte::from_u64(item.size as u64),
        priority: item.priority,
        taken_at: item.taken_at.and_then(|date| {
            item.taken_at_timezone.map(|tz| {
                date.with_timezone(&FixedOffset::east_opt(tz).expect("Invalid timezone offset"))
            })
        }),
        camera_make: item.camera_make,
        camera_model: item.camera_model,
        width: item.width,
        height: item.height,
        last_saved: item.last_saved,
    })
}
