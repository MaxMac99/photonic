use crate::Error;

pub enum MediumError {
    UnsupportedFile,
    MimeMismatch {
        found_mime: String,
        given_mime: String,
    },
    WrongFilename,
    WrongAlbum,
    NoDateTaken,
    UnknownError(String),
}

impl From<meta::Error> for MediumError {
    fn from(value: meta::Error) -> Self {
        match value {
            meta::Error::NoMimeType => MediumError::UnsupportedFile,
            meta::Error::NotSupported(_) => MediumError::UnsupportedFile,
            _ => MediumError::UnsupportedFile,
        }
    }
}

impl From<std::io::Error> for MediumError {
    fn from(value: std::io::Error) -> Self {
        MediumError::UnknownError(value.to_string())
    }
}

impl From<MediumError> for Error {
    fn from(value: MediumError) -> Self {
        match value {
            MediumError::UnsupportedFile => Error::InvalidArgument(String::from("The file is not supported")),
            MediumError::MimeMismatch { found_mime, given_mime } => Error::InvalidArgument(format!("The given mime type ({}) does not match the found mime type ({})", given_mime, found_mime)),
            MediumError::WrongFilename => Error::InvalidArgument(String::from("Could not find extension from filename")),
            MediumError::WrongAlbum => Error::InvalidArgument(String::from("Could not find the album")),
            MediumError::NoDateTaken => Error::InvalidArgument(String::from("Could not find a date when the image was taken")),
            MediumError::UnknownError(value) => Error::Internal(value),
        }
    }
}