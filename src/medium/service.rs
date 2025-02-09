use crate::{
    album,
    error::{
        AlbumNotFoundSnafu, MediumItemNotFoundSnafu, MediumNotFoundSnafu, QuotaExceededSnafu,
        Result,
    },
    exif::MediumItemExifLoadedEvent,
    medium::{
        model::CreateMediumInput,
        repo,
        repo::{MediumDb, MediumItemDb},
        CreateMediumItemInput, FindAllMediaOptions, GetMediumPreviewOptions,
        MediumItemCreatedEvent, MediumItemResponse, MediumItemType, MediumResponse, MediumType,
    },
    state::{AppState, ArcConnection},
    storage,
    storage::StorageLocation,
    user::{service::get_user, UserInput},
};
use byte_unit::Byte;
use chrono::{FixedOffset, Utc};
use futures_util::future::{try_join_all, BoxFuture};
use itertools::Itertools;
use mime::Mime;
use mime_serde_shim::Wrapper;
use snafu::OptionExt;
use std::fs::Metadata;
use tokio::{fs, fs::File, try_join};
use tokio_util::io::ReaderStream;
use tracing::log::{info, warn};
use uuid::Uuid;

#[tracing::instrument(skip(state, conn, tmp_file))]
pub async fn create_medium<F>(
    state: AppState,
    conn: ArcConnection<'_>,
    tmp_file: F,
    filesize: Byte,
    user: UserInput,
    medium_opts: CreateMediumInput,
    medium_item_opts: CreateMediumItemInput,
    mime: Mime,
) -> Result<MediumItemCreatedEvent>
where
    F: FnOnce(ArcConnection<'_>, Uuid) -> BoxFuture<'_, Result<StorageLocation>>,
{
    let user_id = user.sub;
    let usage = get_user(conn.clone(), user_id).await?.quota_used;
    if usage.as_u64() + filesize.as_u64() > user.quota.as_u64() {
        return QuotaExceededSnafu.fail();
    }

    let medium_item_id = Uuid::new_v4();

    let ((tmp_path, metadata), medium) = try_join!(
        store_tmp_file(&state, conn.clone(), tmp_file, medium_item_id),
        save_new_medium(
            conn.clone(),
            user.clone(),
            medium_item_opts.clone(),
            mime.clone(),
            medium_item_id,
            medium_opts.clone(),
            filesize,
        )
    )?;

    if metadata.len() != filesize.as_u64() {
        warn!(
            "File size mismatch: expected {}, got {}",
            filesize,
            metadata.len()
        );
    }

    let medium_item_event = MediumItemCreatedEvent {
        id: medium_item_id,
        medium_id: medium.id,
        medium_item_type: MediumItemType::Original,
        album_id: medium_opts.album_id,
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

    info!("Created medium with id: {}", medium.id);
    Ok(medium_item_event)
}

#[tracing::instrument(skip(state, conn, tmp_file))]
pub async fn add_medium_item<F>(
    state: AppState,
    conn: ArcConnection<'_>,
    tmp_file: F,
    filesize: Byte,
    user: UserInput,
    medium_id: Uuid,
    medium_item_type: MediumItemType,
    medium_item_opts: CreateMediumItemInput,
    mime: Mime,
) -> Result<MediumItemCreatedEvent>
where
    F: FnOnce(ArcConnection<'_>, Uuid) -> BoxFuture<'_, Result<StorageLocation>>,
{
    let user_id = user.sub;
    let usage = get_user(conn.clone(), user_id).await?.quota_used;
    if usage.as_u64() + filesize.as_u64() > user.quota.as_u64() {
        return QuotaExceededSnafu.fail();
    }

    // Check if the owner has the medium
    let medium = repo::find_medium(conn.clone(), medium_id)
        .await?
        .context(MediumNotFoundSnafu { id: medium_id })?;
    if medium.owner_id != user_id {
        return MediumNotFoundSnafu { id: medium_id }.fail();
    }

    if let Some(id) = medium.album_id {
        album::service::find_album_by_id(conn.clone(), id)
            .await?
            .context(AlbumNotFoundSnafu { id })?;
    }

    let medium_item_id = Uuid::new_v4();
    let ((tmp_path, metadata), _) = try_join!(
        store_tmp_file(&state, conn.clone(), tmp_file, medium_item_id),
        save_new_medium_item(
            conn.clone(),
            medium_id,
            medium_item_type,
            medium_item_opts.clone(),
            mime.clone(),
            medium_item_id,
            filesize,
        )
    )?;

    if metadata.len() != filesize.as_u64() {
        warn!(
            "File size mismatch: expected {}, got {}",
            filesize,
            metadata.len()
        );
    }

    let medium_item_event = MediumItemCreatedEvent {
        id: medium_item_id,
        medium_id,
        medium_item_type,
        album_id: medium.album_id,
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

    info!("Added medium item with id: {}", medium_item_id);
    Ok(medium_item_event)
}

#[tracing::instrument(skip(conn))]
pub async fn get_media(
    conn: ArcConnection<'_>,
    user: UserInput,
    opts: FindAllMediaOptions,
) -> Result<Vec<MediumResponse>> {
    let owner_id = user.sub;
    let media = repo::get_media(conn.clone(), owner_id, opts).await?;
    let result = try_join_all(
        media
            .into_iter()
            .map(|medium| async { create_medium_response(medium, conn.clone()).await }),
    )
    .await?;

    Ok(result)
}

#[tracing::instrument(skip(conn))]
pub async fn update_medium_item_from_exif(
    conn: ArcConnection<'_>,
    exif: MediumItemExifLoadedEvent,
) -> Result<()> {
    let mut medium_item = repo::find_medium_item_by_id(conn.clone(), exif.medium_item_id)
        .await?
        .context(MediumItemNotFoundSnafu {
            id: exif.medium_item_id,
        })?;
    medium_item.width = medium_item.width.or(exif.width.map(|width| width as i32));
    medium_item.height = medium_item
        .height
        .or(exif.height.map(|height| height as i32));

    let medium_id = medium_item.medium_id;
    repo::update_medium_item(conn.clone(), medium_item).await?;

    let mut medium = repo::find_medium(conn.clone(), medium_id)
        .await?
        .context(MediumNotFoundSnafu { id: medium_id })?;
    medium.taken_at = medium.taken_at.or(exif.date.map(|date| date.to_utc()));
    medium.taken_at_timezone = medium
        .taken_at_timezone
        .or(exif.date.map(|date| date.offset().local_minus_utc()));
    medium.camera_make = medium.camera_make.or(exif.camera_make);
    medium.camera_model = medium.camera_model.or(exif.camera_model);

    repo::update_medium(conn, medium).await?;

    Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn delete_medium(
    conn: ArcConnection<'_>,
    user: UserInput,
    medium_id: Uuid,
) -> Result<()> {
    repo::delete_medium(conn, user.sub, medium_id).await?;

    info!("Deleted medium with id: {}", medium_id);
    Ok(())
}

#[tracing::instrument(skip(state, conn))]
pub async fn get_raw_medium(
    state: AppState,
    conn: ArcConnection<'_>,
    user: UserInput,
    medium_id: Uuid,
    medium_item_id: Uuid,
) -> Result<(Mime, ReaderStream<File>)> {
    // Check if the owner owns the medium
    let medium = repo::find_medium(conn.clone(), medium_id)
        .await?
        .context(MediumNotFoundSnafu { id: medium_id })?;
    if medium.owner_id != user.sub {
        return MediumNotFoundSnafu { id: medium_id }.fail();
    }

    let medium_item = repo::find_medium_item_by_id(conn.clone(), medium_item_id)
        .await?
        .context(MediumItemNotFoundSnafu { id: medium_item_id })?;
    let location = storage::service::find_fastest_location(conn.clone(), medium_item_id).await?;

    let file = File::open(location.full_path(&state.config.storage)).await?;
    Ok((medium_item.mime.parse().unwrap(), ReaderStream::new(file)))
}

#[tracing::instrument(skip(state, conn))]
pub async fn get_medium_preview(
    state: AppState,
    conn: ArcConnection<'_>,
    user: UserInput,
    medium_id: Uuid,
    opts: GetMediumPreviewOptions,
) -> Result<(Mime, ReaderStream<File>)> {
    // Check if user has access
    let medium = repo::find_medium(conn.clone(), medium_id)
        .await?
        .context(MediumNotFoundSnafu { id: medium_id })?;
    if medium.owner_id != user.sub {
        return MediumNotFoundSnafu { id: medium_id }.fail();
    }

    let medium_items = repo::find_medium_items_by_id(conn.clone(), medium_id)
        .await?
        .into_iter()
        .filter(|item| item.medium_item_type != MediumItemType::Sidecar)
        .sorted_by(|a, b| {
            if matches!(a.medium_item_type, MediumItemType::Preview) {
                return std::cmp::Ordering::Less;
            }
            if matches!(b.medium_item_type, MediumItemType::Preview) {
                return std::cmp::Ordering::Greater;
            }
            if matches!(a.medium_item_type, MediumItemType::Edit) {
                return std::cmp::Ordering::Less;
            }
            if matches!(b.medium_item_type, MediumItemType::Edit) {
                return std::cmp::Ordering::Greater;
            }
            a.priority.cmp(&b.priority)
        })
        .take(1)
        .collect_vec();
    let medium_item = medium_items
        .first()
        .context(MediumNotFoundSnafu { id: medium_id })?;
    let location = storage::service::find_fastest_location(conn.clone(), medium_item.id).await?;

    let file = File::open(location.full_path(&state.config.storage)).await?;
    Ok((medium_item.mime.parse().unwrap(), ReaderStream::new(file)))
}

#[tracing::instrument(skip(state, conn, tmp_file))]
async fn store_tmp_file<F>(
    state: &AppState,
    conn: ArcConnection<'_>,
    tmp_file: F,
    medium_item_id: Uuid,
) -> Result<(StorageLocation, Metadata)>
where
    F: FnOnce(ArcConnection<'_>, Uuid) -> BoxFuture<'_, Result<StorageLocation>>,
{
    let tmp_path = tmp_file(conn, medium_item_id).await?;
    let metadata = fs::metadata(tmp_path.full_path(&state.config.storage)).await?;
    Ok((tmp_path, metadata))
}

#[tracing::instrument(skip(arc_conn))]
async fn save_new_medium(
    arc_conn: ArcConnection<'_>,
    user: UserInput,
    medium_item_opts: CreateMediumItemInput,
    mime: Mime,
    medium_item_id: Uuid,
    medium_opts: CreateMediumInput,
    filesize: Byte,
) -> Result<MediumDb> {
    let medium_type = medium_opts
        .medium_type
        .unwrap_or_else(|| MediumType::from(mime.clone()));
    let medium = MediumDb {
        id: Uuid::new_v4(),
        owner_id: user.sub,
        medium_type,
        leading_item_id: medium_item_id,
        album_id: medium_opts.album_id,
        taken_at: medium_item_opts.date_taken.map(|date| date.to_utc()),
        taken_at_timezone: medium_item_opts
            .date_taken
            .map(|date| date.offset().local_minus_utc()),
        camera_make: medium_item_opts.camera_make.clone(),
        camera_model: medium_item_opts.camera_model.clone(),
    };
    repo::create_medium(arc_conn.clone(), medium.clone()).await?;
    repo::add_tags(arc_conn.clone(), medium.id, medium_opts.tags).await?;
    save_new_medium_item(
        arc_conn,
        medium.id,
        MediumItemType::Original,
        medium_item_opts,
        mime,
        medium_item_id,
        filesize,
    )
    .await?;

    Ok(medium)
}

#[tracing::instrument(skip(conn))]
async fn save_new_medium_item(
    conn: ArcConnection<'_>,
    medium_id: Uuid,
    medium_item_type: MediumItemType,
    medium_item_opts: CreateMediumItemInput,
    mime: Mime,
    medium_item_id: Uuid,
    filesize: Byte,
) -> Result<()> {
    let conn = &mut *conn.get_connection().await;
    repo::create_medium_item(
        conn,
        MediumItemDb {
            id: medium_item_id,
            medium_id,
            medium_item_type,
            mime: mime.to_string(),
            filename: medium_item_opts.filename.clone(),
            size: filesize.as_u64() as i64,
            priority: medium_item_opts.priority.clone(),
            width: None,
            height: None,
            last_saved: Utc::now().naive_utc(),
            deleted_at: None,
        },
    )
    .await?;
    Ok(())
}

#[tracing::instrument(skip(conn))]
async fn create_medium_response(
    medium: MediumDb,
    conn: ArcConnection<'_>,
) -> Result<MediumResponse> {
    let items = repo::find_medium_items_by_id(conn.clone(), medium.id).await?;
    let items = try_join_all(items.into_iter().map(|item| {
        let medium = medium.clone();
        let conn = conn.clone();
        async move { create_medium_item_response(item, medium, conn).await }
    }))
    .await?;
    Ok(MediumResponse {
        id: medium.id,
        medium_type: medium.medium_type,
        album_id: medium.album_id,
        taken_at: medium.taken_at.and_then(|date| {
            medium.taken_at_timezone.map(|tz| {
                date.with_timezone(&FixedOffset::east_opt(tz).expect("Invalid timezone offset"))
            })
        }),
        camera_make: medium.camera_make,
        camera_model: medium.camera_model,
        items,
    })
}

#[tracing::instrument(skip(conn))]
async fn create_medium_item_response(
    item: MediumItemDb,
    medium: MediumDb,
    conn: ArcConnection<'_>,
) -> Result<MediumItemResponse> {
    let locations = storage::service::find_locations_by_medium_item_id(conn, item.id).await?;
    Ok(MediumItemResponse {
        id: item.id,
        is_primary: item.id == medium.leading_item_id,
        medium_item_type: item.medium_item_type,
        mime: item.mime.parse().unwrap(),
        filename: item.filename,
        locations,
        filesize: Byte::from_u64(item.size as u64),
        priority: item.priority,
        width: item.width,
        height: item.height,
        last_saved: item.last_saved,
    })
}
