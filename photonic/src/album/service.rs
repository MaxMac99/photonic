use crate::{
    album::{model::Album, repo::get_album_by_id},
    error::Result,
    server::AppState,
};
use uuid::Uuid;

pub(crate) async fn get_album(
    app_state: AppState,
    album_id: Option<Uuid>,
) -> Result<Option<Album>> {
    let mut album: Option<Album> = None;
    if let Some(album_id) = album_id {
        album = Some(get_album_by_id(&app_state.pool, album_id).await?);
    }
    Ok(album)
}
