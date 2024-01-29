use bson::oid::ObjectId;

use crate::{error::Result, Service};

impl Service {
    pub async fn move_to_trash(&self, id: ObjectId) -> Result<()> {
        self.repo.move_to_trash(id).await
    }
}
