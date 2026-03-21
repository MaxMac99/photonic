mod metadata_extracted;
mod metadata_extraction_failed;
mod metadata_extraction_started;

pub use metadata_extracted::MetadataExtractedEvent;
pub use metadata_extraction_failed::MetadataExtractionFailedEvent;
pub use metadata_extraction_started::MetadataExtractionStartedEvent;

use serde::{Deserialize, Serialize};

use crate::event::{DomainEvent, EventMetadata};

/// Sum type of all events for the Metadata aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MetadataEvent {
    ExtractionStarted(MetadataExtractionStartedEvent),
    Extracted(MetadataExtractedEvent),
    ExtractionFailed(MetadataExtractionFailedEvent),
}

impl DomainEvent for MetadataEvent {
    fn metadata(&self) -> &EventMetadata {
        match self {
            MetadataEvent::ExtractionStarted(e) => e.metadata(),
            MetadataEvent::Extracted(e) => e.metadata(),
            MetadataEvent::ExtractionFailed(e) => e.metadata(),
        }
    }
}

impl From<MetadataExtractionStartedEvent> for MetadataEvent {
    fn from(e: MetadataExtractionStartedEvent) -> Self {
        MetadataEvent::ExtractionStarted(e)
    }
}

impl From<MetadataExtractedEvent> for MetadataEvent {
    fn from(e: MetadataExtractedEvent) -> Self {
        MetadataEvent::Extracted(e)
    }
}

impl From<MetadataExtractionFailedEvent> for MetadataEvent {
    fn from(e: MetadataExtractionFailedEvent) -> Self {
        MetadataEvent::ExtractionFailed(e)
    }
}
