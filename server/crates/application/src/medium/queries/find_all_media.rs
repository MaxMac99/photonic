use std::sync::Arc;

use derive_new::new;
use domain::{
    medium::{MediumFilter, MediumListItem},
    user::UserId,
};
use tracing::{debug, error, info, instrument};

use crate::{error::ApplicationResult, medium::ports::MediumRepository};

#[derive(Debug)]
pub struct FindAllMediaQuery {
    pub user_id: UserId,
    pub filter: MediumFilter,
}

#[derive(new)]
pub struct FindAllMediaHandler {
    medium_repository: Arc<dyn MediumRepository>,
}

impl FindAllMediaHandler {
    #[instrument(skip(self), fields(
        user_id = %query.user_id,
        per_page = query.filter.per_page,
        has_cursor = query.filter.cursor.is_some(),
        has_date_filter = query.filter.start_date.is_some() || query.filter.end_date.is_some(),
        has_album_filter = query.filter.album_id.is_some(),
        has_tags = !query.filter.tags.is_empty()
    ))]
    pub async fn handle(&self, query: FindAllMediaQuery) -> ApplicationResult<Vec<MediumListItem>> {
        info!("Finding all media for user");

        let media = self
            .medium_repository
            .find_all(query.filter, query.user_id)
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to find media");
                e
            })?;

        debug!(count = media.len(), "Media retrieved successfully");

        Ok(media)
    }
}
