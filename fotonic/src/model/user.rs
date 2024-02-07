use bson::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "givenName", skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    pub quota: u64,
    #[serde(rename = "quotaUsed")]
    pub quota_used: u64,
}
