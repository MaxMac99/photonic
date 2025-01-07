use crate::{
    album::{
        model::{AlbumResponse, CreateAlbumInput, FindAllAlbumsOptions},
        repo,
        repo::InsertAlbumDb,
    },
    error::Result,
    state::ArcConnection,
    user::UserInput,
};
use tracing::log::info;
use uuid::Uuid;

#[tracing::instrument(skip(conn))]
pub async fn create_album(
    conn: ArcConnection<'_>,
    user: UserInput,
    opts: CreateAlbumInput,
) -> Result<Uuid> {
    let album_id = Uuid::new_v4();
    repo::create_album(
        conn,
        InsertAlbumDb {
            id: album_id,
            owner_id: user.sub,
            title: opts.title,
            description: opts.description,
        },
    )
    .await?;

    info!("Created album with id: {}", album_id);
    Ok(album_id)
}

#[tracing::instrument(skip(conn))]
pub async fn find_albums(
    conn: ArcConnection<'_>,
    user: UserInput,
    opts: FindAllAlbumsOptions,
) -> Result<Vec<AlbumResponse>> {
    let albums = repo::find_albums(conn, user.sub, opts)
        .await?
        .into_iter()
        .map(|album| AlbumResponse {
            id: album.id,
            title: album.title,
            description: album.description,
            number_of_items: album.number_of_items as u64,
            minimum_date: album.minimum_date,
            maximum_date: album.maximum_date,
        })
        .collect();
    Ok(albums)
}

#[tracing::instrument(skip(conn))]
pub async fn find_album_by_id(
    conn: ArcConnection<'_>,
    album_id: Uuid,
) -> Result<Option<InsertAlbumDb>> {
    let album = repo::get_album(conn, album_id).await?;
    Ok(album)
}
