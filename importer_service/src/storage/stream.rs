use crate::state::AppState;
use axum::{body::Bytes, BoxError};
use common::{
    error::Result,
    stream::events::{StorageLocation, StorageVariant},
};
use futures::{io, Stream, TryStreamExt};
use std::{fs::remove_file, path::PathBuf};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use uuid::Uuid;

pub async fn store_tmp_from_stream<S, E>(
    state: AppState,
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

    Ok(location)
}
