use std::backtrace::Backtrace;

use snafu::Snafu;

use meta::MetaError;

use crate::{model::MediumItemType, ObjectId};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(transparent)]
    MongoDb {
        source: mongodb::error::Error,
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
    FindMediumById { id: ObjectId, backtrace: Backtrace },
    #[snafu(display("Could not find medium item of type {medium_type:?}"))]
    FindMediumItemById {
        medium_type: MediumItemType,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not find album with id {id}"))]
    FindAlbumById { id: ObjectId, backtrace: Backtrace },
    #[snafu(display("The given file is outside the base storage"))]
    OutsideBaseStorage { backtrace: Backtrace },
    #[snafu(display("Could not find a file extension"))]
    NoFileExtension { backtrace: Backtrace },
    #[snafu(display("Could not find the date when this medium was taken"))]
    NoDateTaken { backtrace: Backtrace },
}
