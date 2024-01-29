use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    error::Result,
    model::{DateDirection, Medium},
    service::Service,
    ObjectId,
};

#[derive(Debug, Deserialize)]
pub struct FindAllMediaInput {
    pub per_page: Option<u16>,
    pub page_last_date: Option<DateTime<Utc>>,
    pub page_last_id: Option<ObjectId>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub album_id: Option<ObjectId>,
    pub include_no_album: Option<bool>,
    pub date_direction: Option<DateDirection>,
}

impl Service {
    pub async fn find_all_media(
        &self,
        opts: &FindAllMediaInput,
    ) -> Result<Vec<Medium>> {
        self.repo
            .find_media(
                opts.per_page.unwrap_or(100) as i64,
                opts.page_last_date,
                opts.page_last_id,
                opts.start_date,
                opts.end_date,
                opts.album_id,
                opts.include_no_album.unwrap_or(true),
                opts.date_direction.unwrap_or(DateDirection::NewestFirst),
            )
            .await
    }
}
