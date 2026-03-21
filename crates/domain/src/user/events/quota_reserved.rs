use byte_unit::Byte;
use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::{
    event::{DomainEvent, EventMetadata},
    user::UserId,
};

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

    fn event_type(&self) -> &'static str {
        "QuotaReserved"
    }
}
