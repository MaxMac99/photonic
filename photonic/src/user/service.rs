use crate::{error::Result, user, AppState};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateUserInput {
    pub id: Uuid,
    pub username: Option<String>,
    pub given_name: Option<String>,
    pub email: Option<String>,
    pub quota: u64,
}

pub async fn create_or_update_user(app_state: &AppState, user: CreateUserInput) -> Result<()> {
    user::repo::create_or_update_user(&app_state.db_pool, user).await
}
