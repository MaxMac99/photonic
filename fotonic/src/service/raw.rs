use std::backtrace::Backtrace;

use snafu::{OptionExt, Snafu};

use crate::{model::FileItem, repository::MediumRepoError, ObjectId, Service};

#[derive(Debug, Snafu)]
pub enum RawMediumError {
    #[snafu(display("Could not get medium"), context(false))]
    SaveMedium {
        #[snafu(backtrace)]
        source: MediumRepoError,
    },
    #[snafu(display("Could not find medium with id {medium_id}"))]
    MediumNotFound {
        medium_id: ObjectId,
        backtrace: Backtrace,
    },
    #[snafu(display("Could not find {item:#?} in medium"))]
    ItemNotFound {
        item: MediumFileSubItem,
        backtrace: Backtrace,
    },
}

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
    ) -> Result<FileItem, RawMediumError> {
        let medium = self
            .repo
            .get_medium(medium_id)
            .await?
            .context(MediumNotFoundSnafu { medium_id })?;

        let mut file_item = match medium_file_sub_type {
            MediumFileSubItem::Original(item_id) => medium
                .originals
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file),
            MediumFileSubItem::Edit(item_id) => medium
                .edits
                .into_iter()
                .filter(|item| item.file.id == item_id)
                .next()
                .map(|item| item.file),
            MediumFileSubItem::Preview => medium.preview.map(|item| item.file),
            MediumFileSubItem::Sidecar(item_id) => medium
                .sidecars
                .into_iter()
                .filter(|item| item.id == item_id)
                .next(),
        }
        .context(ItemNotFoundSnafu {
            item: medium_file_sub_type,
        })?;
        file_item.path = self.store.get_full_path(&file_item);

        Ok(file_item)
    }
}
