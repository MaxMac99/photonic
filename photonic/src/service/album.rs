use uuid::Uuid;

impl Service {
    pub(crate) async fn get_album(
        &self,
        album_id: Option<Uuid>,
    ) -> crate::error::Result<Option<Album>> {
        let mut album: Option<Album> = None;
        if let Some(album_id) = album_id {
            album = Some(self.repo.get_album_by_id(album_id).await?);
        }
        Ok(album)
    }
}
