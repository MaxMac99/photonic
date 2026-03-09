use byte_unit::Byte;
use derive_new::new;

use crate::domain::{
    event::{DomainEvent, EventMetadata},
    user::UserId,
};

#[derive(new, Debug, Clone)]
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
}
