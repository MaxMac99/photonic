use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::error::Error;

use crate::models::Album;
use crate::repository::MongoRepo;

impl MongoRepo {
    pub async fn get_album_by_id(&self, id: ObjectId) -> Result<Option<Album>, Error> {
        let result = self.album_col.find_one(
            doc! {
                "_id": id,
            },
            None,
        ).await;
        result
    }
}
