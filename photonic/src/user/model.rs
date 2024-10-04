use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub given_name: Option<String>,
    pub quota: i64,
    pub quota_used: i64,
}
