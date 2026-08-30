use std::{any::Any, path::PathBuf};

use async_trait::async_trait;
use byte_unit::Byte;
use chrono::{DateTime, Utc};
use kernel::{
    app_error::ApplicationResult, error::DomainResult, event_bus::PublishEvent, FileLocation,
    FileMetadata, MediumId, MediumItemId, UserId,
};
use tokio::io::AsyncRead;

use crate::domain::{
    events::{MediumCreatedEvent, MediumUpdatedEvent},
    Medium, MediumFilter, MediumListItem,
};

#[async_trait]
pub trait MediumRepository: Send + Sync {
    async fn find_by_id(&self, id: MediumId, user_id: UserId) -> DomainResult<Option<Medium>>;
    async fn find_all(
        &self,
        filter: MediumFilter,
        user_id: UserId,
    ) -> DomainResult<Vec<MediumListItem>>;
    async fn save(&self, medium: &Medium) -> DomainResult<()>;
    async fn delete(&self, id: MediumId, user_id: UserId) -> DomainResult<()>;
    async fn get_user_usage(&self, user_id: UserId) -> DomainResult<Byte>;
    async fn find_expired_temp_locations(
        &self,
        created_before: DateTime<Utc>,
    ) -> DomainResult<Vec<ExpiredTempLocation>>;
}

pub struct ExpiredTempLocation {
    pub medium_id: MediumId,
    pub item_id: MediumItemId,
    pub owner_id: UserId,
    pub temp_location: FileLocation,
}

/// Opaque handle to an in-flight quota reservation (ADR 0005).
///
/// Minted by a [`QuotaPort`] implementation and only meaningful to the
/// implementation that created it — it carries that implementation's own
/// reservation record.
pub struct Reservation(Box<dyn Any + Send + Sync>);

impl Reservation {
    /// For [`QuotaPort`] implementations only: wrap the implementation's
    /// private reservation record.
    pub fn new(token: Box<dyn Any + Send + Sync>) -> Self {
        Self(token)
    }

    /// For [`QuotaPort`] implementations only: recover the wrapped record.
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        match self.0.downcast::<T>() {
            Ok(boxed) => Ok(*boxed),
            Err(inner) => Err(Self(inner)),
        }
    }
}

/// Quota reservation port (ADR 0005): explicit reserve/commit/release
/// replaces the closure-based API. Reservation and expiry semantics
/// (short-lived soft locks, ADR 0003) live entirely inside the
/// implementation; callers only commit or release the returned handle.
#[async_trait]
pub trait QuotaPort: Send + Sync {
    async fn reserve(&self, user_id: UserId, bytes: Byte) -> ApplicationResult<Reservation>;
    async fn commit(&self, reservation: Reservation) -> ApplicationResult<()>;
    async fn release(&self, reservation: Reservation) -> ApplicationResult<()>;
}

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn store_file(&self, location: &FileLocation, content: Vec<u8>) -> DomainResult<()>;

    async fn store_file_stream(
        &self,
        location: &FileLocation,
        stream: Box<dyn AsyncRead + Send + Unpin>,
    ) -> DomainResult<()>;

    async fn copy_file(&self, src: &FileLocation, dest: &FileLocation) -> DomainResult<()>;
    async fn move_file(&self, src: &FileLocation, dest: &FileLocation) -> DomainResult<()>;
    async fn retrieve_file(&self, location: &FileLocation) -> DomainResult<Vec<u8>>;
    async fn retrieve_file_stream(
        &self,
        location: &FileLocation,
    ) -> DomainResult<Box<dyn AsyncRead + Unpin>>;
    async fn get_local_path(&self, location: &FileLocation) -> DomainResult<PathBuf>;
    async fn delete_file(&self, location: &FileLocation) -> DomainResult<()>;
    async fn get_file_metadata(&self, location: &FileLocation) -> DomainResult<FileMetadata>;
}

pub trait PublishMediumEvent:
    PublishEvent<MediumCreatedEvent> + PublishEvent<MediumUpdatedEvent>
{
}

impl<T> PublishMediumEvent for T where
    T: PublishEvent<MediumCreatedEvent> + PublishEvent<MediumUpdatedEvent>
{
}
