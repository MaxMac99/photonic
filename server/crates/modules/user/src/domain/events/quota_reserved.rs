use byte_unit::Byte;
use derive_new::new;
use kernel::{
    event::{DomainEvent, EventMetadata},
    UserId,
};
use serde::{Deserialize, Serialize};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[new(visibility = "pub(crate)")]
pub struct QuotaReservedEvent {
    pub user_id: UserId,
    pub bytes: Byte,
    pub quota_used_after: Byte,
    #[new(default)]
    pub metadata: EventMetadata,
}

impl DomainEvent for QuotaReservedEvent {
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
