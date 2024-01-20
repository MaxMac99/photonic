use std::sync::Arc;

use snafu::{ResultExt, Whatever};

pub use create_medium::CreateMediumError;
pub use create_medium::CreateMediumInput;

use crate::config::Config;
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
    pub async fn new(config: Arc<Config>) -> Result<Self, Whatever> {
        let repo = Repository::init(config.clone()).await?;
        let store = Store::new(config);
        let meta = meta::Service::new().await
            .whatever_context("Could not init meta Service")?;
        Ok(Self {
            repo,
            store,
            meta,
        })
    }
}
