use byte_unit::Byte;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub sub: Uuid,
    pub quota: Byte,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub preferred_username: Option<String>,
    pub nickname: Option<String>,
    pub groups: Vec<String>,
}

impl User {
    pub fn get_username(&self) -> Option<String> {
        self.preferred_username
            .clone()
            .or_else(|| self.nickname.clone())
            .or_else(|| self.name.clone())
            .or_else(|| self.given_name.clone())
    }
}
