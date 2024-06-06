use crate::{
    error::Error,
    model::FileItem,
    repository::{dto::sidecar::NewSidecar, Repository},
    schema::sidecars,
};
use diesel::{Connection, RunQueryDsl};
use uuid::Uuid;

impl Repository {
    pub async fn add_sidecar(
        &self,
        medium_id: Uuid,
        file_item: FileItem,
    ) -> crate::error::Result<Uuid> {
        let new_sidecar = Self::create_new_sidecar(medium_id, file_item);

        let conn = self.pool.get().await?;
        let res = conn
            .interact(move |conn| {
                conn.transaction(|conn| {
                    let inserted_id: Uuid = diesel::insert_into(sidecars::table)
                        .values(&new_sidecar)
                        .returning(sidecars::id)
                        .get_result(conn)?;
                    Ok::<Uuid, Error>(inserted_id)
                })
            })
            .await??;

        Ok(res)
    }

    pub(crate) fn create_new_sidecar(medium_id: Uuid, file_item: FileItem) -> NewSidecar {
        NewSidecar {
            medium_id,
            mime: file_item.mime.to_string(),
            filename: file_item.filename,
            path: file_item.path.into_os_string().into_string().unwrap(),
            size: file_item.filesize as i64,
            location: file_item.location,
            priority: file_item.priority,
            last_saved: file_item.last_saved,
        }
    }
}
