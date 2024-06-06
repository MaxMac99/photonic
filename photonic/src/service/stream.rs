use crate::{
    error::{Error, Result},
    Service,
};
use axum::{body::Bytes, BoxError};
use futures::{Stream, TryFuture};
use futures_util::{io, TryFutureExt, TryStreamExt};
use std::path::PathBuf;
use tokio::{fs, fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::{debug, error};

impl Service {
    pub(crate) async fn store_stream_temporarily<S, E, T, F, Fut>(
        &self,
        extension: &str,
        stream: S,
        f: F,
    ) -> Result<T>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
        E: Into<BoxError>,
        F: FnOnce(PathBuf) -> Fut,
        Fut: TryFuture<Ok = T, Error = Error>,
    {
        let temp_path = self.store.get_temp_file_path(extension);
        let body_with_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_error);
        futures::pin_mut!(body_reader);

        let file = File::create(&temp_path).await?;
        let mut buffer = BufWriter::new(file);

        tokio::io::copy(&mut body_reader, &mut buffer).await?;

        debug!("Uploaded file temporarily to {}", temp_path.display());

        f(temp_path.clone())
            .or_else(|err| async {
                // Remove file if it could not store metadata
                if let Err(remove_err) = fs::remove_file(&temp_path).await {
                    error!("Could not delete file for rollback: {}", remove_err);
                }
                Err(err)
            })
            .await
    }
}
