use byte_unit::Byte;
use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    user::UserId,
};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct UserCreatedEvent {
    pub user_id: UserId,
    pub username: String,
    pub email: Option<String>,
    pub quota: Byte,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for UserCreatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn event_type(&self) -> &'static str {
        "UserCreated"
    }
}