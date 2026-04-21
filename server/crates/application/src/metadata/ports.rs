use async_trait::async_trait;
use domain::{
    error::DomainResult,
    medium::{FileLocation, MediumId},
    metadata::{
        events::{
            MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
        },
        Metadata, MetadataId,
    },
};

use crate::event_bus::PublishEvent;

#[async_trait]
pub trait MetadataRepository: Send + Sync {
    async fn find_by_id(&self, id: MetadataId) -> DomainResult<Option<Metadata>>;
    async fn find_by_medium_id(&self, medium_id: MediumId) -> DomainResult<Option<Metadata>>;
    async fn save(&self, metadata: &Metadata) -> DomainResult<()>;
    async fn delete(&self, id: MetadataId) -> DomainResult<()>;
}

#[async_trait]
pub trait MetadataExtractor: Send + Sync {
    /// Extract metadata from a file at the given location
    async fn extract(&self, location: &FileLocation, medium_id: MediumId)
        -> DomainResult<Metadata>;
}

pub trait PublishMetadataEvent:
    PublishEvent<MetadataExtractionStartedEvent>
    + PublishEvent<MetadataExtractedEvent>
    + PublishEvent<MetadataExtractionFailedEvent>
{
}

impl<T> PublishMetadataEvent for T where
    T: PublishEvent<MetadataExtractionStartedEvent>
        + PublishEvent<MetadataExtractedEvent>
        + PublishEvent<MetadataExtractionFailedEvent>
{
}
