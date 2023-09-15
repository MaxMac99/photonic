use std::sync::Arc;

pub use path_opts::PathOptions;

use crate::config::Config;

mod path_opts;
mod save;

#[derive(Debug)]
pub struct Store {
    config: Arc<Config>,
}

impl Store {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config
        }
    }
}
