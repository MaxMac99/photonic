use std::sync::Arc;

pub use path::PathOptions;
pub(crate) use save::ImportError;

use crate::config::Config;

mod path;
mod save;

#[derive(Debug)]
pub struct Store {
    config: Arc<Config>,
}

impl Store {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}
