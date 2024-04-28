use uuid::Uuid;

use crate::{error::Result, Service};

impl Service {
    pub async fn move_to_trash(&self, id: Uuid) -> Result<()> {
        self.repo.move_medium_item_to_trash(id).await
    }
}
