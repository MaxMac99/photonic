use chrono::{DateTime, Utc};
use serde::Deserialize;
use snafu::OptionExt;
use uuid::Uuid;

use crate::error::{FindMediumItemByIdSnafu, FindSidecarByIdSnafu, Result};

#[derive(Debug, Deserialize)]
pub struct FindAllMediaInput {
    pub per_page: Option<u16>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub album_id: Option<Uuid>,
    pub show_only_unset_albums: Option<bool>,
    pub date_direction: Option<DateDirection>,
}

#[derive(Debug)]
pub enum GetMediumFileType {
    Original,
    Edit,
    Preview,
    Sidecar,
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

    pub async fn get_medium_file(
        &self,
        user_id: Uuid,
        medium_id: Uuid,
        item_id: Uuid,
        medium_file_type: GetMediumFileType,
    ) -> Result<FileItem> {
        let medium = self.repo.get_medium(medium_id, user_id).await?;

        let mut file_item = match medium_file_type {
            GetMediumFileType::Original => medium
                .originals
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Original,
                }),
            GetMediumFileType::Edit => medium
                .edits
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Edit,
                }),
            GetMediumFileType::Preview => medium
                .previews
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Preview,
                }),
            GetMediumFileType::Sidecar => medium
                .sidecars
                .into_iter()
                .filter(|item| item.id == item_id)
                .next()
                .context(FindSidecarByIdSnafu),
        }?;
        file_item.path = self.store.get_full_path(&file_item);

        Ok(file_item)
    }
}
