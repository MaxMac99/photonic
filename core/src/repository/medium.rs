use mongodb::results::InsertOneResult;

use crate::entities::Medium;
use crate::errors::MediumError;
use crate::repository::Repository;

impl Repository {
    pub async fn create_medium(
        &self,
        new_medium: Medium,
    ) -> Result<InsertOneResult, MediumError> {
        let new_doc = Medium {
            id: None,
            medium_type: new_medium.medium_type,
            date_taken: new_medium.date_taken,
            originals: new_medium.originals,
            album: new_medium.album,
            tags: new_medium.tags,
            preview: new_medium.preview,
            edits: new_medium.edits,
            sidecars: new_medium.sidecars,
        };
        let medium = self
            .medium_col
            .insert_one(new_doc, None)
            .await
            .ok()
            .expect("Error creating user");
        Ok(medium)
    }
}
