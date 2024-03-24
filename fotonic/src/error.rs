use std::backtrace::Backtrace;

use deadpool_diesel::InteractError;
use snafu::Snafu;
use uuid::Uuid;

use meta::MetaError;

use crate::model::MediumItemType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(transparent)]
    Database {
        source: diesel::result::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Deadpool {
        source: deadpool_diesel::PoolError,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Interact {
        source: InteractError,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Io {
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[snafu(transparent)]
    Metadata {
        source: MetaError,
        backtrace: Backtrace,
    },
    // Custom errors
    #[snafu(display("Could not find medium with id {id}"))]
    FindMediumById { id: Uuid, backtrace: Backtrace },
    #[snafu(display("Could not find medium item of type {medium_type:?}"))]
    FindMediumItemById {
        medium_type: MediumItemType,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not find sidecar"))]
    FindSidecarById { backtrace: Backtrace },
    #[snafu(display("Could not find album with id {id}"))]
    FindAlbumById { id: Uuid, backtrace: Backtrace },
    #[snafu(display("Could not find user with id {id}"))]
    FindUserById { id: Uuid, backtrace: Backtrace },
    #[snafu(display("There is not enough quota left"))]
    NoQuotaLeft { backtrace: Backtrace },
    #[snafu(display("The given file is outside the base storage"))]
    OutsideBaseStorage { backtrace: Backtrace },
    #[snafu(display("Could not find a file extension"))]
    NoFileExtension { backtrace: Backtrace },
    #[snafu(display("The given file already exists"))]
    FileAlreadyExists { backtrace: Backtrace },
    #[snafu(display("Could not find the date when this medium was taken"))]
    NoDateTaken { backtrace: Backtrace },
}
