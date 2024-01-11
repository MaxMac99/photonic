use std::fmt;

use exiftool::ExifError;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Could not find Mimetype.
    NoMimeType,
    /// The file is not supported.
    NotSupported(String),
    Unknown(String),
}

impl From<ExifError> for Error {
    fn from(value: ExifError) -> Self {
        match value {
            ExifError::CouldNotFindToolError(text) => Error::Unknown(String::from(text.to_string())),
            ExifError::ParseError(text) => Error::NotSupported(text),
            ExifError::InvalidPathError => Error::Unknown(String::from("Given path not valid")),
        }
    }
}

impl From<exif::Error> for Error {
    fn from(value: exif::Error) -> Self {
        match value {
            exif::Error::InvalidFormat(text) => Error::NotSupported(String::from(text)),
            exif::Error::Io(_) => Error::NotSupported(String::from("IO error")),
            exif::Error::NotFound(text) => Error::NotSupported(String::from(text)),
            exif::Error::BlankValue(text) => Error::NotSupported(String::from(text)),
            exif::Error::TooBig(text) => Error::NotSupported(String::from(text)),
            exif::Error::NotSupported(text) => Error::NotSupported(String::from(text)),
            exif::Error::UnexpectedValue(text) => Error::NotSupported(String::from(text)),
            _ => Error::NotSupported(String::from("Unknown Error"))
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoMimeType => f.write_str("Could not find Mimetype"),
            Error::NotSupported(msg) => f.write_str(msg),
            Error::Unknown(msg) => f.write_str(msg),
        }
    }
}
