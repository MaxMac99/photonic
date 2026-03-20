use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims structure - this is what comes from the OAuth2 provider
/// This is an infrastructure concern, not a domain entity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtUserClaims {
    pub sub: Uuid, // Subject - unique user identifier
    pub email: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub preferred_username: Option<String>,
    pub nickname: Option<String>,

    // Custom claims your OAuth2 provider might include
    pub quota: Option<Byte>,
}

impl JwtUserClaims {
    /// Extract the best available username from claims
    pub fn get_username(&self) -> Option<String> {
        self.preferred_username
            .clone()
            .or_else(|| self.nickname.clone())
            .or_else(|| self.name.clone())
            .or_else(|| self.given_name.clone())
    }

    /// Convert JWT claims to domain user ID
    pub fn user_id(&self) -> Uuid {
        self.sub
    }
}
