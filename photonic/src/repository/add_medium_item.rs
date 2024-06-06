use crate::{
    error::{Error, Result},
    model::{MediumItem, MediumItemType},
    repository::{dto::medium_item::NewMediumItem, Repository},
    schema::medium_items,
};
use diesel::{Connection, RunQueryDsl};
use uuid::Uuid;

impl Repository {
    pub async fn add_medium_item(
        &self,
        medium_id: Uuid,
        item_type: MediumItemType,
        medium_item: MediumItem,
    ) -> Result<Uuid> {
        let new_medium_item = Self::create_new_medium_item(medium_id, item_type, medium_item);

        let conn = self.pool.get().await?;
        let res = conn
            .interact(move |conn| {
                conn.transaction(|conn| {
                    let inserted_id: Uuid = diesel::insert_into(medium_items::table)
                        .values(&new_medium_item)
                        .returning(medium_items::id)
                        .get_result(conn)?;
                    Ok::<Uuid, Error>(inserted_id)
                })
            })
            .await??;

        Ok(res)
    }

    pub(crate) fn create_new_medium_item(
        medium_id: Uuid,
        item_type: MediumItemType,
        medium_item: MediumItem,
    ) -> NewMediumItem {
        NewMediumItem {
            medium_id,
            medium_item_type: item_type,
            mime: medium_item.file.mime.to_string(),
            filename: medium_item.file.filename,
            path: medium_item
                .file
                .path
                .into_os_string()
                .into_string()
                .unwrap(),
            size: medium_item.file.filesize as i64,
            location: medium_item.file.location,
            priority: medium_item.file.priority,
            timezone: medium_item.date_taken.timezone().local_minus_utc(),
            taken_at: medium_item.date_taken.naive_utc(),
            last_saved: medium_item.file.last_saved,
            width: medium_item.width as i32,
            height: medium_item.height as i32,
        }
    }
}
