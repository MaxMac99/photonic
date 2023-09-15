use std::sync::Arc;

use crate::config::Config;
use crate::repository::Repository;
use crate::store::Store;

mod create_medium;
pub mod inputs;

#[derive(Debug)]
pub struct Service {
    config: Arc<Config>,
    repo: Repository,
    store: Store,
}

impl Service {
    pub async fn new(config: Arc<Config>) -> Self {
        let repo = Repository::init().await;
        let store = Store::new(config.clone());
        Self {
            config,
            repo,
            store,
        }
    }
}
