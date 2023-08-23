use std::path::PathBuf;

pub use path_opts::PathOptions;

pub mod error;
mod path_opts;
mod save;
mod repo;

pub struct PhotoRepo {
    base_path: PathBuf,
    pattern: String,
}

