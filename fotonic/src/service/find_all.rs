use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::Result,
    model::{DateDirection, Medium},
    service::Service,
};

#[derive(Debug, Deserialize)]
pub struct FindAllMediaInput {
    pub per_page: Option<u16>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub album_id: Option<Uuid>,
    pub show_only_unset_albums: Option<bool>,
    pub date_direction: Option<DateDirection>,
}

impl Service {
    pub async fn find_all_media(
        &self,
        user_id: Uuid,
        opts: &FindAllMediaInput,
    ) -> Result<Vec<Medium>> {
        self.repo
            .find_media(
                user_id,
                opts.per_page.unwrap_or(100) as i64,
                opts.start_date,
                opts.end_date,
                opts.album_id,
                opts.show_only_unset_albums.unwrap_or(false),
                opts.date_direction.unwrap_or(DateDirection::NewestFirst),
            )
            .await
    }
}
