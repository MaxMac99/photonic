mod quota_committed;
mod quota_released;
mod quota_reserved;
mod user_created;
mod user_updated;

pub use quota_committed::QuotaCommittedEvent;
pub use quota_released::QuotaReleasedEvent;
pub use quota_reserved::QuotaReservedEvent;
pub use user_created::UserCreatedEvent;
pub use user_updated::UserUpdatedEvent;
pub(super) use user_updated::UserUpdatedEventBuilder;

use serde::{Deserialize, Serialize};

use crate::event::{DomainEvent, EventMetadata};

/// Sum type of all events for the User aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserEvent {
    UserCreated(UserCreatedEvent),
    UserUpdated(UserUpdatedEvent),
    QuotaReserved(QuotaReservedEvent),
    QuotaCommitted(QuotaCommittedEvent),
    QuotaReleased(QuotaReleasedEvent),
}

impl DomainEvent for UserEvent {
    fn metadata(&self) -> &EventMetadata {
        match self {
            UserEvent::UserCreated(e) => e.metadata(),
            UserEvent::UserUpdated(e) => e.metadata(),
            UserEvent::QuotaReserved(e) => e.metadata(),
            UserEvent::QuotaCommitted(e) => e.metadata(),
            UserEvent::QuotaReleased(e) => e.metadata(),
        }
    }

    fn event_type(&self) -> &'static str {
        match self {
            UserEvent::UserCreated(e) => e.event_type(),
            UserEvent::UserUpdated(e) => e.event_type(),
            UserEvent::QuotaReserved(e) => e.event_type(),
            UserEvent::QuotaCommitted(e) => e.event_type(),
            UserEvent::QuotaReleased(e) => e.event_type(),
        }
    }
}

impl From<UserCreatedEvent> for UserEvent {
    fn from(e: UserCreatedEvent) -> Self {
        UserEvent::UserCreated(e)
    }
}

impl From<UserUpdatedEvent> for UserEvent {
    fn from(e: UserUpdatedEvent) -> Self {
        UserEvent::UserUpdated(e)
    }
}

impl From<QuotaReservedEvent> for UserEvent {
    fn from(e: QuotaReservedEvent) -> Self {
        UserEvent::QuotaReserved(e)
    }
}

impl From<QuotaCommittedEvent> for UserEvent {
    fn from(e: QuotaCommittedEvent) -> Self {
        UserEvent::QuotaCommitted(e)
    }
}

impl From<QuotaReleasedEvent> for UserEvent {
    fn from(e: QuotaReleasedEvent) -> Self {
        UserEvent::QuotaReleased(e)
    }
}
