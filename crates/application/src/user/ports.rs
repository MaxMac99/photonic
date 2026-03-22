use async_trait::async_trait;
use domain::{
    error::DomainResult,
    user::{events::UserEvent, User, UserId},
};

use crate::event_bus::PublishEvent;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> DomainResult<Option<User>>;
    async fn insert(&self, user: &User) -> DomainResult<()>;
    async fn update(&self, user: &User) -> DomainResult<()>;
}

pub trait PublishUserEvent: PublishEvent<UserEvent> {}

impl<T> PublishUserEvent for T where T: PublishEvent<UserEvent> {}
