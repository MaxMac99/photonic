use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Error,
};

use crate::{entities::Album, repository::Repository};

impl Repository {
    pub async fn get_album_by_id(
        &self,
        id: ObjectId,
    ) -> Result<Option<Album>, Error> {
        let result = self
            .album_col
            .find_one(
                doc! {
                    "_id": id,
                },
                None,
            )
            .await;
        result
    }
}
