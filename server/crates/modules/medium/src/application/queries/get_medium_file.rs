use std::sync::Arc;

use derive_new::new;
use kernel::{
    app_error::ApplicationResult, error::EntityNotFoundSnafu, FileLocation, MediumId, MediumItemId,
    UserId,
};
use mime::Mime;
use snafu::OptionExt;
use tokio::io::AsyncRead;
use tracing::{debug, error, instrument};

use crate::{
    application::{ports::FileStorage, queries::MediumQueryPort},
    domain::{MediumItem, MediumItemType, ThumbnailVariant},
};

/// A file ready to be streamed to the client.
pub struct MediumFile {
    pub mime: Mime,
    pub filesize: u64,
    pub stream: Box<dyn AsyncRead + Send + Unpin>,
}

#[derive(Debug)]
pub struct GetMediumItemFileQuery {
    pub user_id: UserId,
    pub medium_id: MediumId,
    pub item_id: MediumItemId,
}

#[derive(Debug)]
pub struct GetMediumPreviewFileQuery {
    pub user_id: UserId,
    pub medium_id: MediumId,
    /// Longest edge requested by the client, if any. Used to pick the
    /// smallest preview variant that still satisfies the request.
    pub requested_edge: Option<u32>,
}

#[derive(new)]
pub struct GetMediumFileHandler {
    query_port: Arc<dyn MediumQueryPort>,
    file_storage: Arc<dyn FileStorage>,
}

impl GetMediumFileHandler {
    #[instrument(skip(self), fields(
        user_id = %query.user_id,
        medium_id = %query.medium_id,
        item_id = %query.item_id,
    ))]
    pub async fn get_item_file(
        &self,
        query: GetMediumItemFileQuery,
    ) -> ApplicationResult<MediumFile> {
        let medium = self
            .query_port
            .find_by_id(query.medium_id, query.user_id)
            .await?
            .context(EntityNotFoundSnafu {
                entity: "Medium",
                id: query.medium_id,
            })?;

        let item = medium
            .items
            .iter()
            .find(|i| i.id == query.item_id)
            .context(EntityNotFoundSnafu {
                entity: "MediumItem",
                id: query.item_id,
            })?;

        debug!(item_id = %item.id, "Retrieving medium item file");
        self.retrieve(item).await
    }

    #[instrument(skip(self), fields(
        user_id = %query.user_id,
        medium_id = %query.medium_id,
        requested_edge = ?query.requested_edge,
    ))]
    pub async fn get_preview_file(
        &self,
        query: GetMediumPreviewFileQuery,
    ) -> ApplicationResult<MediumFile> {
        let medium = self
            .query_port
            .find_by_id(query.medium_id, query.user_id)
            .await?
            .context(EntityNotFoundSnafu {
                entity: "Medium",
                id: query.medium_id,
            })?;

        let needed = query
            .requested_edge
            .unwrap_or_else(|| ThumbnailVariant::Small.max_dimension());
        let item = choose_preview(&medium.items, needed).context(EntityNotFoundSnafu {
            entity: "Preview",
            id: query.medium_id,
        })?;

        debug!(item_id = %item.id, "Retrieving medium preview");
        self.retrieve(item).await
    }

    async fn retrieve(&self, item: &MediumItem) -> ApplicationResult<MediumFile> {
        let location = pick_best_location(&item.locations).context(EntityNotFoundSnafu {
            entity: "FileLocation",
            id: item.id,
        })?;

        let stream = self
            .file_storage
            .retrieve_file_stream(location)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to retrieve file from storage");
                kernel::app_error::ApplicationError::Domain { source: e }
            })?;

        Ok(MediumFile {
            mime: item.mime.clone(),
            filesize: item.filesize.as_u64(),
            stream,
        })
    }
}

/// The smallest preview whose variant still covers `needed` (longest edge
/// in pixels); falls back to the largest available preview.
fn choose_preview(items: &[MediumItem], needed: u32) -> Option<&MediumItem> {
    let mut previews: Vec<&MediumItem> = items
        .iter()
        .filter(|i| i.medium_item_type == MediumItemType::Preview)
        .collect();

    previews.sort_by_key(|i| derived_variant_limit(i));

    previews
        .iter()
        .find(|i| derived_variant_limit(i) >= needed)
        .copied()
        .or_else(|| previews.last().copied())
}

fn derived_variant_limit(item: &MediumItem) -> u32 {
    item.dimensions
        .as_ref()
        .map(|d| ThumbnailVariant::from_dimensions(d.width(), d.height()).max_dimension())
        .unwrap_or(0)
}

/// Prefers the fastest available storage tier (Permanent > Cache > Temporary).
fn pick_best_location(locations: &[FileLocation]) -> Option<&FileLocation> {
    locations.iter().min_by_key(|l| l.storage_tier.speed())
}
