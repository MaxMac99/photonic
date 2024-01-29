use mongodb::bson::{doc, oid::ObjectId};
use snafu::OptionExt;

use crate::{
    error::{FindAlbumByIdSnafu, Result},
    model::Album,
    repository::Repository,
};

impl Repository {
    pub async fn get_album_by_id(&self, id: ObjectId) -> Result<Album> {
        self.album_col
            .find_one(
                doc! {
                    "_id": id,
                },
                None,
            )
            .await?
            .context(FindAlbumByIdSnafu { id })
    }
}
