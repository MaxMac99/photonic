use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOptions;
use mongodb::results::InsertOneResult;
use snafu::{ResultExt, Snafu};

use crate::entities::Medium;
use crate::ObjectId;
use crate::repository::Repository;

#[derive(Snafu, Debug)]
pub enum SaveMediumError {
    #[snafu(display("Could not create medium"))]
    Insert { source: mongodb::error::Error },
    #[snafu(display("Could not find media"))]
    Find { source: mongodb::error::Error },
    #[snafu(display("Could not collect media"))]
    Collect { source: mongodb::error::Error },
}

impl Repository {
    pub async fn create_medium(&self, new_medium: Medium) -> Result<InsertOneResult, SaveMediumError> {
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
        };
        let medium = self.medium_col
            .insert_one(new_doc, None)
            .await
            .context(InsertSnafu)?;
        Ok(medium)
    }

    pub async fn find_media(&self, page_size: i64, next_date: Option<DateTime<Utc>>, next_id: Option<ObjectId>, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>, album_id: Option<ObjectId>, include_no_album: bool) -> Result<Vec<Medium>, SaveMediumError> {
        let find_opts = FindOptions::builder()
            .limit(page_size)
            .sort(doc! {
                "date_taken": -1,
                "_id": -1,
            })
            .build();

        let mut filters: Vec<Document> = Vec::new();
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
        let cursor = self.medium_col
            .find(filter, find_opts)
            .await
            .context(FindSnafu)?;

        let result: Vec<Medium> = cursor.try_collect()
            .await
            .context(CollectSnafu)?;

        Ok(result)
    }
}
