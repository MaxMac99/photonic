use thiserror::Error;

pub(crate) use medium::MediumError;

mod medium;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Internal error")]
    Internal(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Authentication required.")]
    AuthenticationRequired,
    #[error("{0}")]
    PermissionDenied(String),
    #[error("{0}")]
    InvalidArgument(String),
    #[error("{0}")]
    AlreadyExists(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Internal(value.to_string())
    }
}

impl From<std::env::VarError> for Error {
    fn from(value: std::env::VarError) -> Self {
        match value {
            std::env::VarError::NotPresent => Error::NotFound("Env var not found".into()),
            _ => Error::Internal(value.to_string()),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::InvalidArgument(err.to_string())
    }
}

impl From<std::str::ParseBoolError> for Error {
    fn from(err: std::str::ParseBoolError) -> Self {
        Error::InvalidArgument(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::Internal(err.to_string())
    }
}
