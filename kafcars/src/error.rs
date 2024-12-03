use schema_registry_converter::error::SRCError;
use snafu::Snafu;
use std::backtrace::Backtrace;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(transparent)]
    SchemaRegistry {
        source: SRCError,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not serialize key to string"))]
    KeyNotString,
}
