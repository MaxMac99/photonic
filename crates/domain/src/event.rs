use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
        }
    }
}

pub trait DomainEvent: Send + Sync + Clone + Debug {
    fn metadata(&self) -> &EventMetadata;

    /// Discriminator string for event type routing (e.g., "MediumCreated")
    fn event_type(&self) -> &'static str;
}
