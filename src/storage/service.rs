use crate::{
    error::Result,
    exif::MediumItemExifLoadedEvent,
    medium::MediumItemCreatedEvent,
    state::{AppState, Transaction},
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
use std::{fs::remove_file, path::PathBuf};
use tokio::{fs, fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::log::debug;
use uuid::Uuid;

pub async fn find_locations_by_medium_item_id(
    transaction: &mut Transaction,
    medium_item_id: Uuid,
) -> Result<Vec<StorageLocation>> {
    repo::find_locations_by_medium_item_id(transaction, medium_item_id).await
}

pub async fn store_tmp_from_stream<S, E>(
    state: AppState,
    transaction: &mut Transaction,
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

    repo::add_storage_location(transaction, medium_item_id, location.clone()).await?;

    Ok(location)
}

pub async fn move_medium_item_to_permanent(
    state: AppState,
    transaction: &mut Transaction,
    created: MediumItemCreatedEvent,
    exif: Option<MediumItemExifLoadedEvent>,
) -> Result<()> {
    let user = get_user(transaction, created.user).await?;
    let date_taken = created
        .date_taken
        .or(exif.clone().map(|e| e.date).flatten().clone());
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
    fs::rename(old_path, &new_path).await?;

    repo::move_location(
        transaction,
        created.id,
        created.location,
        new_location.clone(),
    )
    .await?;

    state
        .event_bus
        .publish(MediumItemMovedEvent {
            id: created.id,
            new_location,
        })
        .await?;

    Ok(())
}
