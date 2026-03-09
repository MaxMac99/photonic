use byte_unit::Byte;
use derive_builder::Builder;

use crate::domain::{
    event::{DomainEvent, EventMetadata},
    user::UserId,
};

/// Event published when user profile is updated (username, email, or quota limit)
#[derive(Debug, Clone, Builder)]
#[builder(setter(into, strip_option))]
pub struct UserUpdatedEvent {
    pub user_id: UserId,
    #[builder(default)]
    pub old_username: Option<String>,
    #[builder(default)]
    pub new_username: Option<String>,
    #[builder(default)]
    pub old_email: Option<String>,
    #[builder(default)]
    pub new_email: Option<String>,
    #[builder(default)]
    pub old_quota: Option<Byte>,
    #[builder(default)]
    pub new_quota: Option<Byte>,
    #[builder(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for UserUpdatedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
