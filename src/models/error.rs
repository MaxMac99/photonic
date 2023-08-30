use actix_web::error;
use actix_web::http::StatusCode;
use derive_more::Display;
use mime::Mime;

use crate::common::meta::MetaError;
use crate::repository::photo::error::PhotoError;

#[derive(Debug, Display)]
pub enum AppError {
    /// Given mime type is not equal to the extracted mime type
    #[display(fmt = "Found mime type {} in file but received {}", file, content_type)]
    WrongType {
        file: Mime,
        content_type: Mime,
    },
    /// Error while reading meta information
    #[display(fmt = "Error reading meta information: {}", _0)]
    MetaInformationError(&'static str),
    /// The filename needs to contain the file extension
    #[display(fmt = "Given filename was not in correct format")]
    WrongFilename,
    /// Could not extract date taken
    #[display(fmt = "Could not find a date for the image")]
    NoDateTaken,
    /// Could not find album with id
    #[display(fmt = "Could not find album with id {}", _0)]
    AlbumNotFound(String),
    /// The image already exists
    #[display(fmt = "The image already exists")]
    AlreadyExists,
    /// There is something wrong with the configuration
    #[display(fmt = "Something is wrong with the configuration")]
    ConfigurationError,
    /// There was an unknown error,
    #[display(fmt = "There was an unknown error")]
    UnknownError,
}

impl error::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::WrongType { .. } => StatusCode::BAD_REQUEST,
            AppError::MetaInformationError(_) => StatusCode::BAD_REQUEST,
            AppError::WrongFilename => StatusCode::BAD_REQUEST,
            AppError::NoDateTaken => StatusCode::BAD_REQUEST,
            AppError::AlreadyExists => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<PhotoError> for AppError {
    fn from(error: PhotoError) -> Self {
        match error {
            PhotoError::WrongBase => AppError::WrongFilename,
            PhotoError::NoExtension => AppError::WrongFilename,
            PhotoError::NoPermission => AppError::ConfigurationError,
            PhotoError::AlreadyExists => AppError::AlreadyExists,
            PhotoError::FileCreationError { .. } => AppError::UnknownError,
        }
    }
}

impl From<MetaError> for AppError {
    fn from(error: MetaError) -> Self {
        match error {
            MetaError::NoMimeType => AppError::MetaInformationError("Could not extract mime type"),
            MetaError::NotSupported(content) => AppError::MetaInformationError(content),
        }
    }
}
