use std::sync::Arc;

use crate::config::Config;
use crate::Error;
use crate::repository::Repository;
use crate::store::Store;

mod create_medium;
pub mod inputs;

#[derive(Debug)]
pub struct Service {
    repo: Repository,
    store: Store,
    meta: meta::Service,
}

impl Service {
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        let repo = Repository::init(config.clone()).await?;
        let store = Store::new(config);
        let meta = meta::Service::new().await?;
        Ok(Self {
            repo,
            store,
            meta,
        })
    }
}
