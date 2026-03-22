use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    /// The expected aggregate version before this event is applied.
    /// Used for optimistic concurrency in the event store.
    pub expected_version: i64,
}

impl EventMetadata {
    pub fn new(expected_version: i64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            expected_version,
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            expected_version: 0,
        }
    }
}

pub trait DomainEvent: Send + Sync + Clone + Debug {
    fn metadata(&self) -> &EventMetadata;
}
