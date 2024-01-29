use snafu::OptionExt;

use crate::{
    error::{FindMediumItemByIdSnafu, Result},
    model::{FileItem, MediumItemType},
    ObjectId, Service,
};

#[derive(Debug)]
pub enum MediumFileSubItem {
    Original(ObjectId),
    Edit(ObjectId),
    Preview,
    Sidecar(ObjectId),
}

impl Service {
    pub async fn get_medium_file(
        &self,
        medium_id: ObjectId,
        medium_file_sub_type: MediumFileSubItem,
    ) -> Result<FileItem> {
        let medium = self.repo.get_medium(medium_id).await?;

        let mut file_item = match medium_file_sub_type {
            MediumFileSubItem::Original(item_id) => medium
                .originals
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Original(item_id),
                }),
            MediumFileSubItem::Edit(item_id) => medium
                .edits
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Edit(item_id),
                }),
            MediumFileSubItem::Preview => medium
                .preview
                .map(|item| item.file)
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Preview,
                }),
            MediumFileSubItem::Sidecar(item_id) => medium
                .sidecars
                .into_iter()
                .filter(|item| item.id == item_id)
                .next()
                .context(FindMediumItemByIdSnafu {
                    medium_type: MediumItemType::Sidecar(item_id),
                }),
        }?;
        file_item.path = self.store.get_full_path(&file_item);

        Ok(file_item)
    }
}
