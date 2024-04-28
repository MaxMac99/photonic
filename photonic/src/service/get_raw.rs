use snafu::OptionExt;
use uuid::Uuid;

use crate::{
    error::{FindMediumItemByIdSnafu, FindSidecarByIdSnafu, Result},
    model::{FileItem, MediumItemType},
    Service,
};

#[derive(Debug)]
pub enum GetMediumFileType {
    Original,
    Edit,
    Preview,
    Sidecar,
}

impl Service {
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
