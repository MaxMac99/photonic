use crate::{
    album,
    error::{AlbumNotFoundSnafu, Result, StorageVariantNotFoundSnafu},
    exif::MediumItemExifLoadedEvent,
    medium::MediumItemCreatedEvent,
    state::{AppState, ArcConnection},
    storage::{
        events::MediumItemMovedEvent,
        pattern::{create_path, PatternFields},
        repo, StorageLocation, StorageVariant,
    },
    user::service::get_user,
};
use axum::BoxError;
use bytes::Bytes;
use chrono::Datelike;
use futures::Stream;
use futures_util::{io, TryStreamExt};
use snafu::OptionExt;
use std::{fs::remove_file, path::PathBuf};
use tokio::{fs, fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::log::{debug, info};
use uuid::Uuid;

#[tracing::instrument(skip(conn))]
pub async fn find_locations_by_medium_item_id(
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
) -> Result<Vec<StorageLocation>> {
    repo::find_locations_by_medium_item_id(conn, medium_item_id).await
}

#[tracing::instrument(skip(state, conn, stream))]
pub async fn store_tmp_from_stream<S, E>(
    state: AppState,
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
    stream: S,
    extension: String,
) -> Result<StorageLocation>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let location = StorageLocation {
        variant: StorageVariant::Temp,
        path: PathBuf::from(format!("{}.{}", Uuid::new_v4().as_hyphenated(), extension)),
    };
    let destination = location.full_path(&state.config.storage);
    let body_with_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_error);
    futures::pin_mut!(body_reader);

    let file = File::create(&destination).await?;
    let mut buffer = BufWriter::new(file);

    tokio::io::copy(&mut body_reader, &mut buffer)
        .await
        .or_else(|err| {
            remove_file(&destination).expect("Failed to remove file after error");
            Err(err)
        })?;

    repo::add_storage_location(conn, medium_item_id, location.clone()).await?;

    info!("Stored file temporarily at {:?}", destination);

    Ok(location)
}

#[tracing::instrument(skip(state, conn))]
pub async fn copy_medium_item_to_permanent(
    state: AppState,
    conn: ArcConnection<'_>,
    created: MediumItemCreatedEvent,
    exif: Option<MediumItemExifLoadedEvent>,
) -> Result<MediumItemMovedEvent> {
    let user = get_user(conn.clone(), created.user).await?;
    let date_taken = created
        .date_taken
        .or(exif.clone().map(|e| e.date).flatten().clone());
    if let Some(id) = created.album_id {
        album::service::find_album_by_id(conn.clone(), id)
            .await?
            .context(AlbumNotFoundSnafu { id })?;
    };

    let fields = PatternFields {
        filename: Some(created.filename),
        extension: created.extension,
        user: Some(user.username),
        date: date_taken,
        camera_make: created
            .camera_make
            .or(exif.clone().map(|e| e.camera_make).flatten()),
        camera_model: created
            .camera_model
            .or(exif.clone().map(|e| e.camera_model).flatten()),
        album: None,
        album_year: date_taken.map(|d| d.year()),
    };
    let new_path = create_path(&state.config.storage.pattern, fields);
    let new_location = StorageLocation {
        variant: StorageVariant::Originals,
        path: new_path.clone().into(),
    };
    let old_path = created.location.full_path(&state.config.storage);
    let new_path = new_location.full_path(&state.config.storage);

    debug!("Moving file from {:?} to {:?}", old_path, new_path);
    fs::create_dir_all(&new_path.parent().expect("Could not get parent dir")).await?;
    fs::copy(old_path.clone(), &new_path).await?;

    repo::add_storage_location(conn, created.id, new_location.clone()).await?;

    info!("Moved file from {:?} to {:?}", old_path, new_path);

    Ok(MediumItemMovedEvent {
        id: created.id,
        new_location,
    })
}

#[tracing::instrument(skip(state, conn))]
pub async fn remove_medium_item_variant(
    state: AppState,
    conn: ArcConnection<'_>,
    medium_item_id: Uuid,
    variant: StorageVariant,
) -> Result<()> {
    let location =
        repo::find_location_variant_by_medium_item_id(conn.clone(), medium_item_id, variant)
            .await?
            .context(StorageVariantNotFoundSnafu {
                medium_item_id,
                variant,
            })?;
    let path = location.full_path(&state.config.storage);
    remove_file(path.clone())?;
    repo::remove_location(conn, medium_item_id, location).await?;

    info!("Removed file at {:?}", path);
    Ok(())
}
