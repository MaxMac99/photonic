use byte_unit::Byte;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    events::{
        QuotaCommittedEvent, QuotaReleasedEvent, QuotaReservedEvent, UserCreatedEvent, UserEvent,
        UserUpdatedEvent, UserUpdatedEventBuilder,
    },
    quota::QuotaState,
};
use crate::{
    aggregate::{AggregateRoot, AggregateVersion},
    error::DomainResult,
};

pub type UserId = Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub version: i64,
    pub username: String,
    pub email: Option<String>,
    pub quota: QuotaState,
}

impl AggregateRoot for User {
    type Event = UserEvent;

    fn aggregate_type() -> &'static str {
        "User"
    }

    fn version(&self) -> AggregateVersion {
        self.version
    }

    fn apply(&mut self, event: &UserEvent) {
        match event {
            UserEvent::UserCreated(e) => {
                self.id = e.user_id;
                self.username = e.username.clone();
                self.email = e.email.clone();
            }
            UserEvent::UserUpdated(e) => {
                if let Some(ref username) = e.new_username {
                    self.username = username.clone();
                }
                if let Some(ref email) = e.new_email {
                    self.email = Some(email.clone());
                }
                if let Some(new_quota) = e.new_quota {
                    self.quota = QuotaState::new_unchecked(self.quota.used(), new_quota);
                }
            }
            UserEvent::QuotaReserved(e) => {
                let _ = self.quota.reserve_quota(e.bytes);
            }
            UserEvent::QuotaCommitted(_) => {
                // Quota is already reserved, commit is a no-op on state
            }
            UserEvent::QuotaReleased(e) => {
                self.quota.release_quota(e.bytes);
            }
        }
        self.version += 1;
    }
}

impl User {
    pub fn new(
        request: UserCreateRequest,
        quota_max_limit: Byte,
    ) -> DomainResult<(Self, UserCreatedEvent)> {
        let user = Self {
            id: request.sub,
            version: 1,
            username: request.username.clone(),
            email: request.email.clone(),
            quota: QuotaState::new(Byte::from_u64(0), request.quota, quota_max_limit)?,
        };

        let event =
            UserCreatedEvent::new(request.sub, request.username, request.email, request.quota);

        Ok((user, event))
    }

    pub fn reserve_quota(&mut self, bytes: Byte) -> DomainResult<QuotaReservedEvent> {
        self.quota.reserve_quota(bytes)?;
        Ok(QuotaReservedEvent::new(self.id, bytes, self.quota.used()))
    }

    pub fn commit_quota(&self, reserved: &QuotaReservedEvent) -> QuotaCommittedEvent {
        QuotaCommittedEvent::new(self.id, reserved.bytes, reserved.metadata.event_id)
    }

    pub fn release_quota(&mut self, reserved: &QuotaReservedEvent) -> QuotaReleasedEvent {
        self.quota.release_quota(reserved.bytes);
        QuotaReleasedEvent::new(
            self.id,
            reserved.bytes,
            self.quota.used(),
            reserved.metadata.event_id,
        )
    }

    pub fn update(
        &mut self,
        request: UserUpdateRequest,
        quota_max_limit: Byte,
    ) -> DomainResult<Option<UserUpdatedEvent>> {
        let mut builder = UserUpdatedEventBuilder::default();
        builder.user_id(self.id);

        let mut changed = false;

        if let Some(username) = request.username {
            if self.username != username {
                builder.old_username(self.username.clone());
                builder.new_username(username.clone());
                self.username = username;
                changed = true;
            }
        }

        if let Some(email) = request.email {
            if self.email.as_ref() != Some(&email) {
                if let Some(old_email) = &self.email {
                    builder.old_email(old_email.clone());
                }
                builder.new_email(email.clone());
                self.email = Some(email);
                changed = true;
            }
        }

        if let Some(new_quota) = request.quota {
            if self.quota.limit() != new_quota {
                builder.old_quota(self.quota.limit());
                builder.new_quota(new_quota);
                self.quota = QuotaState::new(self.quota.used(), new_quota, quota_max_limit)?;
                changed = true;
            }
        }

        if changed {
            Ok(Some(
                builder.build().expect("Failed to build UserUpdatedEvent"),
            ))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserCreateRequest {
    pub sub: UserId,
    pub username: String,
    pub email: Option<String>,
    pub quota: Byte,
}

#[derive(Debug, Clone, Builder)]
pub struct UserUpdateRequest {
    #[builder(setter(into, strip_option), default)]
    pub username: Option<String>,
    #[builder(setter(into), default)]
    pub email: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub quota: Option<Byte>,
}
