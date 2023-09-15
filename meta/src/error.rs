use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Could not find Mimetype.
    NoMimeType,
    /// The file is not supported.
    NotSupported(&'static str),
}

impl From<exif::Error> for Error {
    fn from(value: exif::Error) -> Self {
        match value {
            exif::Error::InvalidFormat(text) => Error::NotSupported(text),
            exif::Error::Io(_) => Error::NotSupported("IO error"),
            exif::Error::NotFound(text) => Error::NotSupported(text),
            exif::Error::BlankValue(text) => Error::NotSupported(text),
            exif::Error::TooBig(text) => Error::NotSupported(text),
            exif::Error::NotSupported(text) => Error::NotSupported(text),
            exif::Error::UnexpectedValue(text) => Error::NotSupported(text),
            _ => Error::NotSupported("Unknown Error")
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoMimeType => f.write_str("Could not extract mime type."),
            Error::NotSupported(msg) => f.write_str(msg),
        }
    }
}
