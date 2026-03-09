use std::path::PathBuf;

use async_trait::async_trait;
use byte_unit::Byte;
use tokio::io::AsyncRead;

use chrono::{DateTime, Utc};

use crate::{
    application::event_bus::PublishEvent,
    domain::{
        error::DomainResult,
        medium::{
            events::{MediumCreatedEvent, MediumItemCreatedEvent, MediumUpdatedEvent},
            FileLocation, FileMetadata, Medium, MediumFilter, MediumId, MediumItemId,
            MediumListItem,
        },
        user::UserId,
    },
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
    PublishEvent<MediumCreatedEvent> + PublishEvent<MediumItemCreatedEvent> + PublishEvent<MediumUpdatedEvent>
{
}

impl<T> PublishMediumEvent for T where
    T: PublishEvent<MediumCreatedEvent> + PublishEvent<MediumItemCreatedEvent> + PublishEvent<MediumUpdatedEvent>
{
}
