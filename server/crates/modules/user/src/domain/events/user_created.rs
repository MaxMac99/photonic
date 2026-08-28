use byte_unit::Byte;
use derive_new::new;
use kernel::{
    event::{DomainEvent, EventMetadata},
    UserId,
};
use serde::{Deserialize, Serialize};

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
}
