use std::sync::Arc;

use derive_new::new;
use tracing::{error, info};

use crate::{
    application::{
        error::ApplicationResult,
        metadata::ports::{MetadataExtractor, MetadataRepository, PublishMetadataEvent},
    },
    domain::{
        medium::{FileLocation, MediumId, MediumItemId},
        metadata::{
            events::{
                MetadataExtractedEvent, MetadataExtractionFailedEvent,
                MetadataExtractionStartedEvent,
            },
            Metadata,
        },
        user::UserId,
    },
};

pub struct ExtractMetadataCommand {
    pub medium_id: MediumId,
    pub leading_item_id: MediumItemId,
    pub user_id: UserId,
    pub file_location: FileLocation,
}

#[derive(new)]
pub struct ExtractMetadataHandler {
    metadata_extractor: Arc<dyn MetadataExtractor>,
    metadata_repository: Arc<dyn MetadataRepository>,
    event_publisher: Arc<dyn PublishMetadataEvent>,
}

impl ExtractMetadataHandler {
    pub async fn handle(&self, command: ExtractMetadataCommand) -> ApplicationResult<()> {
        info!(
            "Starting metadata extraction for medium_id={}, medium_item_id={}",
            command.medium_id, command.leading_item_id
        );

        self.event_publisher
            .publish(MetadataExtractionStartedEvent::new(
                command.medium_id,
                command.leading_item_id,
                command.user_id,
            ))
            .await?;

        match self.extract_metadata(&command).await {
            Ok(metadata) => {
                self.event_publisher
                    .publish(MetadataExtractedEvent::new(
                        command.medium_id,
                        command.leading_item_id,
                        command.user_id,
                        metadata,
                    ))
                    .await?;

                info!(
                    "Metadata extraction completed for medium_id={}, medium_item_id={}",
                    command.medium_id, command.leading_item_id,
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("{:?}", e);

                self.event_publisher
                    .publish(MetadataExtractionFailedEvent::new(
                        command.medium_id,
                        command.leading_item_id,
                        command.user_id,
                        error_msg.clone(),
                    ))
                    .await?;

                error!(
                    "Metadata extraction failed for medium_id={}, medium_item_id={}: {}",
                    command.medium_id, command.leading_item_id, error_msg
                );

                Err(e)
            }
        }
    }

    async fn extract_metadata(
        &self,
        command: &ExtractMetadataCommand,
    ) -> ApplicationResult<Metadata> {
        let metadata = self
            .metadata_extractor
            .extract(&command.file_location, command.medium_id)
            .await?;

        self.metadata_repository.save(&metadata).await?;

        Ok(metadata)
    }
}
