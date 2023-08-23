use std::io::{Error, ErrorKind};

use derive_more::{Display, Error};

use crate::repository::photo::error::PhotoError::{AlreadyExists, FileCreationError, NoPermission};

#[derive(Debug, Display, Error)]
pub enum PhotoError {
    #[display(fmt = "The given path is not path of the base directory.")]
    WrongBase,
    #[display(fmt = "No given file extension")]
    NoExtension,
    #[display(fmt = "No permission")]
    NoPermission,
    #[display(fmt = "File already exists")]
    AlreadyExists,
    #[display(fmt = "Could not create file at \"{}\": {}", path, error)]
    FileCreationError { path: String, error: Error },
}

impl PhotoError {
    pub fn from(path: String, error: Error) -> Self {
        match error.kind() {
            ErrorKind::PermissionDenied => NoPermission,
            ErrorKind::AlreadyExists => AlreadyExists,
            _ => FileCreationError { path, error },
        }
    }
}
