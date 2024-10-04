use crate::{
    error::{Error, Result},
    store::{path::get_temp_file_path, Transaction},
    Config,
};
use axum::{body::Bytes, BoxError};
use futures::{Stream, TryFuture};
use futures_util::{io, TryFutureExt, TryStreamExt};
use std::{path::PathBuf, sync::Arc};
use tokio::{fs, fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::{debug, error};

pub async fn store_stream_temporarily<'a, S, E>(
    transaction: &mut Transaction<'a>,
    config: &Arc<Config>,
    extension: &str,
    stream: S,
) -> Result<PathBuf>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let temp_path = get_temp_file_path(config, extension);
    let body_with_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_error);
    futures::pin_mut!(body_reader);

    let file = File::create(&temp_path).await?;
    let mut buffer = BufWriter::new(file);

    tokio::io::copy(&mut body_reader, &mut buffer).await?;
    transaction.add_rollback(|| async {
        // Remove file if it could not store metadata
        if let Err(remove_err) = fs::remove_file(&temp_path).await {
            error!("Could not delete file for rollback: {}", remove_err);
            return Err(Error::from(remove_err));
        }
        Ok(())
    });

    debug!("Uploaded file temporarily to {}", temp_path.display());
    Ok(temp_path)
}
