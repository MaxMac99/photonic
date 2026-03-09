use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::{QuotaState, User, UserId};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct UserDb {
    pub id: Uuid,
    pub version: i64,
    pub username: Option<String>,
    pub email: Option<String>,
    pub quota: i64,
    pub quota_used: i64,
}

// Reconstitute User from database - no validation, trust DB state
impl From<UserDb> for User {
    fn from(val: UserDb) -> Self {
        let username = val.username.unwrap_or_else(|| {
            tracing::error!("NULL username in database for user {}", val.id);
            "INVALID_USER".to_string()
        });

        User {
            id: UserId::from(val.id),
            version: val.version,
            username,
            email: val.email,
            quota: QuotaState::new_unchecked(
                Byte::from_i64(val.quota_used).expect("invalid quota used"),
                Byte::from_i64(val.quota).expect("invalid quota"),
            ),
        }
    }
}
