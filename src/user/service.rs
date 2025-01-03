use crate::{
    error::Result,
    state::Transaction,
    user::{repo, repo::find_user_by_id, User, UserInput},
};
use uuid::Uuid;

pub async fn create_or_update_user(transaction: &mut Transaction, user: UserInput) -> Result<()> {
    repo::create_or_update_user(transaction, user).await
}

pub async fn get_user(transaction: &mut Transaction, user_id: Uuid) -> Result<User> {
    Ok(find_user_by_id(transaction, user_id).await?)
}
