use uuid::Uuid;

use crate::{error::Result, repository::Repository};

impl Repository {
    pub async fn move_to_trash(&self, id: Uuid) -> Result<()> {
        Ok(())
    }
}
