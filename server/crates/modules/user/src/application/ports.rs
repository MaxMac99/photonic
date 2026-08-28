use async_trait::async_trait;
use kernel::{error::DomainResult, event_bus::PublishEvent, UserId};

use crate::domain::{
    events::{
        QuotaCommittedEvent, QuotaReleasedEvent, QuotaReservedEvent, UserCreatedEvent,
        UserUpdatedEvent,
    },
    User,
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> DomainResult<Option<User>>;
    async fn insert(&self, user: &User) -> DomainResult<()>;
    async fn update(&self, user: &User) -> DomainResult<()>;
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
