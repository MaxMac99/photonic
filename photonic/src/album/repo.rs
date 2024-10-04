use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{
    album::model::Album,
    error::{NotImplementedSnafu, Result},
};

pub async fn get_album_by_id<'e, E>(executor: E, id: Uuid) -> Result<Album>
where
    E: PgExecutor<'e>,
{
    // sqlx::query_as!(Album, "SELECT * FROM albums WHERE id = $1", id)
    //     .fetch_one(executor)
    //     .await
    //     .into()
    Err(NotImplementedSnafu.build())
}
