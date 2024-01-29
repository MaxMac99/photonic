use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    results::InsertOneResult,
};
use snafu::OptionExt;

use crate::{
    error::{FindMediumByIdSnafu, Result},
    model::{DateDirection, Medium},
    repository::Repository,
    ObjectId,
};

impl Repository {
    pub async fn create_medium(
        &self,
        new_medium: Medium,
    ) -> Result<InsertOneResult> {
        let new_doc = Medium {
            id: None,
            medium_type: new_medium.medium_type,
            date_taken: new_medium.date_taken,
            timezone: new_medium.timezone,
            originals: new_medium.originals,
            album: new_medium.album,
            tags: new_medium.tags,
            preview: new_medium.preview,
            edits: new_medium.edits,
            sidecars: new_medium.sidecars,
            additional_data: new_medium.additional_data,
        };
        let medium = self.medium_col.insert_one(new_doc, None).await?;
        Ok(medium)
    }

    pub async fn find_media(
        &self,
        page_size: i64,
        last_date: Option<DateTime<Utc>>,
        last_id: Option<ObjectId>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        album_id: Option<ObjectId>,
        include_no_album: bool,
        date_direction: DateDirection,
    ) -> Result<Vec<Medium>> {
        let direction_val = match date_direction {
            DateDirection::NewestFirst => -1,
            DateDirection::OldestFirst => 1,
        };
        let direction_key = match date_direction {
            DateDirection::NewestFirst => "$lt",
            DateDirection::OldestFirst => "$gt",
        };
        let find_opts = FindOptions::builder()
            .limit(page_size)
            .sort(doc! {
                "date_taken": direction_val,
                "_id": -1,
            })
            .build();

        let mut filters: Vec<Document> = Vec::new();
        if let Some(last_date) = last_date {
            filters.push(doc! {
                "date_taken": {
                    direction_key: last_date,
                }
            });
        }
        if let Some(last_id) = last_id {
            filters.push(doc! {
                "_id": {
                    "$lt": last_id,
                }
            });
        }
        if let Some(start_date) = start_date {
            filters.push(doc! {
                "date_taken": {
                    "$gte": start_date,
                }
            });
        }
        if let Some(start_date) = start_date {
            filters.push(doc! {
                "date_taken": {
                    "$gte": start_date,
                }
            });
        }
        if let Some(end_date) = end_date {
            filters.push(doc! {
                "date_taken": {
                    "$lte": end_date,
                }
            });
        }
        if let Some(album_id) = album_id {
            if include_no_album {
                filters.push(doc! {
                    "$or": [{
                        "album": {
                            "$eq": album_id,
                        },
                    }, {
                        "$not": {
                            "$exists": "album",
                        }
                    }]
                })
            } else {
                filters.push(doc! {
                    "album_id": {
                        "$eq": album_id,
                    }
                })
            }
        }

        let filter: Option<Document> = if filters.is_empty() {
            None
        } else {
            Some(doc! {
                "$and": filters
            })
        };
        let cursor = self.medium_col.find(filter, find_opts).await?;

        let result: Vec<Medium> = cursor.try_collect().await?;

        Ok(result)
    }

    pub async fn get_medium(&self, id: ObjectId) -> Result<Medium> {
        self.medium_col
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await?
            .context(FindMediumByIdSnafu { id })
    }
}
