use std::sync::Arc;

use snafu::{ResultExt, Whatever};

pub use add_medium_item::{AddMediumItemInput, AddMediumItemType};
pub use create_medium::CreateMediumInput;
pub use find_medium::{FindAllMediaInput, GetMediumFileType};
pub use user::CreateUserInput;

use crate::{config::Config, repository::Repository, store::Store};

mod add_medium_item;
mod album;
mod create_medium;
mod find_medium;
mod move_to_trash;
mod path;
mod stream;
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
