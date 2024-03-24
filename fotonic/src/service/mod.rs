use std::sync::Arc;

use snafu::{ResultExt, Whatever};

pub use create_medium::CreateMediumInput;
pub use find_all::FindAllMediaInput;
pub use raw::MediumFileSubItem;
pub use user::CreateUserInput;

use crate::{config::Config, repository::Repository, store::Store};

mod create_medium;
mod find_all;
mod move_to_trash;
mod raw;
mod user;

pub struct Service {
    repo: Repository,
    store: Store,
    meta: meta::Service,
}

impl Service {
    pub async fn new(config: Arc<Config>) -> Result<Self, Whatever> {
        let repo = Repository::init(config.clone()).await?;
        let store = Store::new(config);
        let meta = meta::Service::new()
            .await
            .whatever_context("Could not init meta Service")?;
        Ok(Self { repo, store, meta })
    }
}
