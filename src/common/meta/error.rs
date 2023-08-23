use std::fmt;

use exif::Error;

#[derive(Debug)]
#[non_exhaustive]
pub enum MetaError {
    /// Could not find Mimetype.
    NoMimeType,
    /// The file is not supported.
    NotSupported(&'static str),
}

impl From<Error> for MetaError {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidFormat(text) => MetaError::NotSupported(text),
            Error::Io(_) => MetaError::NotSupported("IO error"),
            Error::NotFound(text) => MetaError::NotSupported(text),
            Error::BlankValue(text) => MetaError::NotSupported(text),
            Error::TooBig(text) => MetaError::NotSupported(text),
            Error::NotSupported(text) => MetaError::NotSupported(text),
            Error::UnexpectedValue(text) => MetaError::NotSupported(text),
            _ => MetaError::NotSupported("Unknown Error")
        }
    }
}

impl fmt::Display for MetaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetaError::NoMimeType => f.write_str("Could not extract mime type."),
            MetaError::NotSupported(msg) => f.write_str(msg),
        }
    }
}
