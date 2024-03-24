use snafu::OptionExt;
use uuid::Uuid;

use crate::{
    error::{FindMediumItemByIdSnafu, FindSidecarByIdSnafu, Result},
    model::{FileItem, MediumItemType},
    Service,
};

#[derive(Debug)]
pub enum MediumFileSubItem {
    Original(Uuid),
    Edit(Uuid),
    Preview,
    Sidecar(Uuid),
}

impl Service {
    pub async fn get_medium_file(
        &self,
        user_id: Uuid,
        medium_id: Uuid,
        medium_file_sub_type: MediumFileSubItem,
    ) -> Result<FileItem> {
        let medium = self.repo.get_medium(medium_id, user_id).await?;

        let mut file_item = match medium_file_sub_type {
            MediumFileSubItem::Original(item_id) => medium
                .originals
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Original,
                }),
            MediumFileSubItem::Edit(item_id) => medium
                .edits
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Edit,
                }),
            MediumFileSubItem::Preview => {
                medium
                    .preview
                    .map(|item| item.file)
                    .context(FindMediumItemByIdSnafu {
                        medium_type: MediumItemType::Preview,
                    })
            }
            MediumFileSubItem::Sidecar(item_id) => medium
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
