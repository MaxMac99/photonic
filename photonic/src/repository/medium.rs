use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Utc};
use diesel::{
    BelongingToDsl, Connection, debug_query, ExpressionMethods, GroupedBy, pg::sql_types,
    PgConnection, QueryDsl, RunQueryDsl, SelectableHelper, sql_query, sql_types::Timestamptz,
};
use itertools::Itertools;
use snafu::OptionExt;
use tracing::debug;
use uuid::Uuid;

use crate::{
    error::{Error, FindMediumByIdSnafu, Result},
    model::{DateDirection, FileItem, Medium, MediumItem, MediumItemType},
    repository::{
        dto::{
            medium,
            medium::NewMedium,
            medium_item,
            medium_item::NewMediumItem,
            sidecar,
            sidecar::{NewSidecar, Sidecar},
        },
        Repository,
    },
    schema::{media, medium_items, sidecars, tags},
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

    pub async fn add_sidecar(&self, medium_id: Uuid, file_item: FileItem) -> Result<Uuid> {
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

    pub async fn find_media(
        &self,
        user_id: Uuid,
        page_size: i64,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        album_id: Option<Uuid>,
        show_only_unset_albums: bool,
        date_direction: DateDirection,
    ) -> Result<Vec<Medium>> {
        // Join first match only of medium_items with the newest date
        let mut query = sql_query(
            "SELECT * FROM media m JOIN (SELECT DISTINCT ON (mis.medium_id) * FROM medium_items mis ORDER BY mis.medium_id, mis.taken_at DESC) AS mi ON mi.medium_id = m.id WHERE m.owner_id = $1",
        )
            .bind::<sql_types::Uuid, _>(user_id)
            .into_boxed();

        let mut pos = 2;
        if let Some(start_date) = start_date {
            query = query
                .sql(format!(" WHERE mi.taken_at >= ${}", pos))
                .bind::<Timestamptz, _>(start_date);
            pos += 1;
        }
        if let Some(end_date) = end_date {
            query = query
                .sql(format!(" WHERE mi.taken_at <= ${}", pos))
                .bind::<Timestamptz, _>(end_date);
            pos += 1;
        }
        if let Some(album_id) = album_id {
            query = query
                .sql(format!(" WHERE m.album_id = ${}", pos))
                .bind::<sql_types::Uuid, _>(album_id);
            pos += 1;
        } else if show_only_unset_albums {
            query = query.sql(" WHERE m.album_id IS NULL");
        }
        query = match date_direction {
            DateDirection::NewestFirst => query.sql(" ORDER BY mi.taken_at DESC"),
            DateDirection::OldestFirst => query.sql(" ORDER BY mi.taken_at ASC"),
        };
        query = query
            .sql(format!(" LIMIT ${}", pos))
            .bind::<diesel::sql_types::Int8, _>(page_size);

        let conn = self.pool.get().await?;
        conn.interact(move |conn| {
            conn.transaction(|conn| {
                debug!("Query: {:?}", debug_query(&query));
                let media = query.load::<medium::Medium>(conn)?;
                let combined = Self::fetch_and_map_medium(media, conn)?;
                Ok::<Vec<Medium>, Error>(combined)
            })
        })
        .await?
    }

    pub async fn get_medium(&self, id: Uuid, user_id: Uuid) -> Result<Medium> {
        let conn = self.pool.get().await?;
        conn.interact(move |conn| {
            conn.transaction(|conn| {
                let media = media::table
                    .filter(media::owner_id.eq(user_id))
                    .filter(media::id.eq(id))
                    .limit(1)
                    .select(medium::Medium::as_select())
                    .load::<medium::Medium>(conn)?;
                let combined = Self::fetch_and_map_medium(media, conn)?
                    .into_iter()
                    .next()
                    .context(FindMediumByIdSnafu { id })?;
                Ok::<Medium, Error>(combined)
            })
        })
        .await?
    }

    fn create_new_medium_item(
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

    fn create_new_sidecar(medium_id: Uuid, file_item: FileItem) -> NewSidecar {
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

    fn fetch_and_map_medium(
        media: Vec<medium::Medium>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Medium>> {
        let medium_items = medium_item::MediumItem::belonging_to(&media)
            .select(medium_item::MediumItem::as_select())
            .load(conn)?;
        let sidecars_iter = sidecar::Sidecar::belonging_to(&media)
            .select(Sidecar::as_select())
            .load::<Sidecar>(conn)?
            .grouped_by(&media)
            .into_iter();
        let tags_iter = medium::MediumTag::belonging_to(&media)
            .inner_join(tags::table)
            .select((medium::MediumTag::as_select(), medium::Tag::as_select()))
            .load::<(medium::MediumTag, medium::Tag)>(conn)?
            .grouped_by(&media)
            .into_iter()
            .map(|items| items.into_iter().map(|(_, tag)| tag.title).collect());
        let combined = medium_items
            .grouped_by(&media)
            .into_iter()
            .zip(sidecars_iter)
            .zip(tags_iter)
            .zip(media)
            .map(|(((items, sidecars), tags), medium)| (medium, items, sidecars, tags))
            .map(|(medium, items, sidecars, tags)| Self::map_medium(medium, items, sidecars, tags))
            .collect::<Vec<Medium>>();
        Ok(combined)
    }

    fn map_medium(
        medium: medium::Medium,
        items: Vec<medium_item::MediumItem>,
        sidecars: Vec<Sidecar>,
        tags: Vec<String>,
    ) -> Medium {
        let mut grouped = items
            .into_iter()
            .map(|item| (item.medium_item_type.clone(), item))
            .into_group_map();
        Medium {
            id: medium.id,
            owner: medium.owner_id,
            medium_type: medium.medium_type,
            originals: grouped
                .remove(&MediumItemType::Original)
                .unwrap_or(vec![])
                .into_iter()
                .map(Self::map_medium_item)
                .collect(),
            album: medium.album_id,
            tags,
            previews: grouped
                .remove(&MediumItemType::Preview)
                .unwrap_or(vec![])
                .into_iter()
                .map(Self::map_medium_item)
                .collect(),
            edits: grouped
                .remove(&MediumItemType::Edit)
                .unwrap_or(vec![])
                .into_iter()
                .map(Self::map_medium_item)
                .collect(),
            sidecars: sidecars.into_iter().map(Self::map_sidecar).collect(),
        }
    }

    fn map_medium_item(item: medium_item::MediumItem) -> MediumItem {
        MediumItem {
            file: FileItem {
                id: item.id,
                mime: item.mime.parse().unwrap(),
                filename: item.filename,
                path: PathBuf::from(item.path),
                filesize: item.size as u64,
                priority: item.priority,
                last_saved: item.last_saved,
                location: item.location,
            },
            width: item.width as u32,
            height: item.height as u32,
            date_taken: item
                .taken_at
                .and_local_timezone(FixedOffset::east_opt(item.timezone).unwrap())
                .unwrap(),
        }
    }

    fn map_sidecar(sidecar: Sidecar) -> FileItem {
        FileItem {
            id: sidecar.id,
            mime: sidecar.mime.parse().unwrap(),
            filename: sidecar.filename,
            path: PathBuf::from(sidecar.path),
            filesize: sidecar.size as u64,
            priority: sidecar.priority,
            last_saved: sidecar.last_saved,
            location: sidecar.location,
        }
    }
}
