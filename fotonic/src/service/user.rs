use bson::Uuid;

use crate::{error::Result, Service};

#[derive(Debug)]
pub struct CreateUserInput {
    pub id: Uuid,
    pub username: Option<String>,
    pub given_name: Option<String>,
    pub email: Option<String>,
    pub quota: u64,
}

impl Service {
    pub async fn create_or_update_user(&self, user: CreateUserInput) -> Result<()> {
        self.repo.create_or_update_user(user).await
    }
}
