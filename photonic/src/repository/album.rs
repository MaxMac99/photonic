use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use crate::{
    error::Result,
    model::Album,
    repository::{dto, Repository},
    schema::albums,
};

impl Repository {
    pub async fn get_album_by_id(&self, id: Uuid) -> Result<Album> {
        let conn = self.pool.get().await?;
        let album = conn
            .interact(move |conn| {
                albums::table
                    .filter(albums::id.eq(id))
                    .select(dto::album::Album::as_select())
                    .get_result(conn)
            })
            .await??;
        Ok(Album {
            id,
            owner: album.owner_id,
            name: album.name,
            description: album.description,
            first_date: None,
            last_date: None,
            title_medium: album.title_medium,
            media: vec![],
        })
    }
}
