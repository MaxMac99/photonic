use diesel::{Connection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    model::{Medium, MediumItemType},
    repository::{
        dto::{medium::NewMedium, medium_item::NewMediumItem, sidecar::NewSidecar},
        Repository,
    },
    schema::{media, medium_items, sidecars},
};

impl Repository {
    pub async fn create_medium(&self, new_medium: Medium) -> Result<Uuid> {
        let new_doc = NewMedium {
            owner_id: new_medium.owner.into(),
            medium_type: new_medium.medium_type,
            album_id: new_medium.album.into(),
        };

        let conn = self.pool.get().await?;
        let res = conn
            .interact(move |conn| {
                conn.transaction(|conn| {
                    let inserted_medium_id: Uuid = diesel::insert_into(media::table)
                        .values(&new_doc)
                        .returning(media::id)
                        .get_result(conn)?;

                    let medium_items = new_medium
                        .originals
                        .into_iter()
                        .map(|medium_item| {
                            Self::create_new_medium_item(
                                inserted_medium_id,
                                MediumItemType::Original,
                                medium_item,
                            )
                        })
                        .chain(new_medium.edits.into_iter().map(|medium_item| {
                            Self::create_new_medium_item(
                                inserted_medium_id,
                                MediumItemType::Edit,
                                medium_item,
                            )
                        }))
                        .chain(new_medium.previews.into_iter().map(|medium_item| {
                            Self::create_new_medium_item(
                                inserted_medium_id,
                                MediumItemType::Preview,
                                medium_item,
                            )
                        }))
                        .collect::<Vec<NewMediumItem>>();

                    diesel::insert_into(medium_items::table)
                        .values(medium_items)
                        .execute(conn)?;

                    diesel::insert_into(sidecars::table)
                        .values(
                            new_medium
                                .sidecars
                                .into_iter()
                                .map(|file_item| {
                                    Self::create_new_sidecar(inserted_medium_id, file_item)
                                })
                                .collect::<Vec<NewSidecar>>(),
                        )
                        .execute(conn)?;

                    Ok::<Uuid, Error>(inserted_medium_id)
                })
            })
            .await??;

        Ok(res)
    }
}
