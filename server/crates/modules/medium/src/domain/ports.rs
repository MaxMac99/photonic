use async_trait::async_trait;
use kernel::error::DomainResult;

use super::{GeneratedThumbnail, Medium, MediumItem};

/// Port for server-side thumbnail generation (issue #1, future work).
///
/// The MVP relies on client-generated thumbnails uploaded via
/// `POST /medium/{id}/item/preview?variant=...`. This port exists so a
/// server-side generator (e.g. a libvips adapter for directory scanning and
/// imports) can be plugged in later without touching the medium module.
///
/// Implemented by adapters; wired via the composition root.
#[async_trait]
pub trait ThumbnailGenerator: Send + Sync {
    /// Generates all configured thumbnail variants for the given medium's
    /// original item.
    async fn generate(
        &self,
        medium: &Medium,
        original: &MediumItem,
    ) -> DomainResult<Vec<GeneratedThumbnail>>;
}
