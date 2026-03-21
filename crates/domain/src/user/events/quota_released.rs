use byte_unit::Byte;
use derive_new::new;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    event::{DomainEvent, EventMetadata},
    user::UserId,
};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct QuotaReleasedEvent {
    pub user_id: UserId,
    pub bytes: Byte,
    pub quota_used_after: Byte,
    pub reserved_event_id: Uuid,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for QuotaReleasedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
