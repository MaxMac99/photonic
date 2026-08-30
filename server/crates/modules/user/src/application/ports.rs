use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::{error::DomainResult, event_bus::PublishEvent, UserId};
use uuid::Uuid;

use crate::domain::{
    events::{
        QuotaCommittedEvent, QuotaReleasedEvent, QuotaReservedEvent, UserCreatedEvent,
        UserUpdatedEvent,
    },
    QuotaReservation, User,
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> DomainResult<Option<User>>;
    async fn insert(&self, user: &User) -> DomainResult<()>;
    async fn update(&self, user: &User) -> DomainResult<()>;
}

/// Persistence for short-lived quota reservations (ADR 0003).
#[async_trait]
pub trait QuotaReservationStore: Send + Sync {
    async fn insert(&self, reservation: &QuotaReservation) -> DomainResult<()>;

    /// Deletes the reservation, returning whether it existed. Claim
    /// semantics: exactly one caller ever sees `true`, so a reservation is
    /// committed/released (or swept) at most once.
    async fn claim(&self, id: Uuid) -> DomainResult<bool>;

    /// All reservations whose expiry has passed (expires_at <= cutoff).
    async fn find_expired(&self, cutoff: DateTime<Utc>) -> DomainResult<Vec<QuotaReservation>>;
}

pub trait PublishUserEvent:
    PublishEvent<UserCreatedEvent>
    + PublishEvent<UserUpdatedEvent>
    + PublishEvent<QuotaReservedEvent>
    + PublishEvent<QuotaCommittedEvent>
    + PublishEvent<QuotaReleasedEvent>
{
}

impl<T> PublishUserEvent for T where
    T: PublishEvent<UserCreatedEvent>
        + PublishEvent<UserUpdatedEvent>
        + PublishEvent<QuotaReservedEvent>
        + PublishEvent<QuotaCommittedEvent>
        + PublishEvent<QuotaReleasedEvent>
{
}
