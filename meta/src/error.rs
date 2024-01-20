use std::backtrace::Backtrace;

use snafu::Snafu;

use exiftool::ExifError;

#[derive(Snafu, Debug)]
pub enum MetaError {
    #[snafu(transparent)]
    Exiftool {
        #[snafu(backtrace)]
        source: ExifError
    },
    #[snafu(display("Could not extract mimetype"), visibility(pub (crate)))]
    ExtractMimetype { backtrace: Backtrace },
    #[snafu(display("Error getting exif information"), context(false))]
    Exif { source: exif::Error, backtrace: Backtrace },
}
