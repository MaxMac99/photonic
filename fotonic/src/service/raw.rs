use std::backtrace::Backtrace;

use snafu::{OptionExt, Snafu};

use crate::{
    model::MediumItem, repository::MediumRepoError, ObjectId, Service,
};

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
    #[snafu(display("Could not find original with id {item_id}"))]
    ItemNotFound {
        item_id: ObjectId,
        backtrace: Backtrace,
    },
}

impl Service {
    pub async fn get_medium_original(
        &self,
        medium_id: ObjectId,
        item_id: ObjectId,
    ) -> Result<MediumItem, RawMediumError> {
        let medium = self
            .repo
            .get_medium(medium_id)
            .await?
            .context(MediumNotFoundSnafu { medium_id })?;

        let mut original = medium
            .originals
            .into_iter()
            .filter(|original| original.id == item_id)
            .next()
            .context(ItemNotFoundSnafu { item_id })?;
        original.path = self.store.get_full_path(&original);

        Ok(original)
    }

    pub async fn get_medium_edit(
        &self,
        medium_id: ObjectId,
        item_id: ObjectId,
    ) -> Result<MediumItem, RawMediumError> {
        let medium = self
            .repo
            .get_medium(medium_id)
            .await?
            .context(MediumNotFoundSnafu { medium_id })?;

        let mut edit = medium
            .edits
            .into_iter()
            .filter(|edit| edit.id == item_id)
            .next()
            .context(ItemNotFoundSnafu { item_id })?;
        edit.path = self.store.get_full_path(&edit);

        Ok(edit)
    }
}
