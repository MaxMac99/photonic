use crate::{state::AppState, user};
use byte_unit::Byte;
use common::{error::Result, user::User};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateUserInput {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub quota: Byte,
}

impl From<User> for CreateUserInput {
    fn from(value: User) -> Self {
        Self {
            id: value.sub,
            username: value.name,
            email: value.email,
            quota: value.quota,
        }
    }
}

pub async fn create_or_update_user(app_state: &AppState, user: CreateUserInput) -> Result<()> {
    user::repo::create_or_update_user(&app_state.db_pool, user).await
}
