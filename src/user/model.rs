use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct UserInput {
    pub sub: Uuid,
    pub quota: Byte,
    pub email: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub preferred_username: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub quota: Byte,
    pub quota_used: Byte,
}

impl UserInput {
    pub fn get_username(&self) -> Option<String> {
        self.preferred_username
            .clone()
            .or_else(|| self.nickname.clone())
            .or_else(|| self.name.clone())
            .or_else(|| self.given_name.clone())
    }
}
