use chrono::{DateTime, Utc};
use snafu::ensure;
use uuid::Uuid;

use super::MediumId;
use crate::{
    domain::error::{DomainResult, ValidationSnafu},
    shared::value_objects::{KeysetCursor, SortDirection},
};

/// Filter for querying media
/// Encapsulates all query criteria and pagination settings
#[derive(Debug, Clone, PartialEq)]
pub struct MediumFilter {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub per_page: u64,
    pub cursor: Option<KeysetCursor<MediumId>>,
    pub tags: Vec<String>,
    pub album_id: Option<Uuid>,
    pub direction: SortDirection,
    pub include_no_album: bool,
}

impl MediumFilter {
    const MIN_PER_PAGE: u64 = 1;
    const MAX_PER_PAGE: u64 = 100;
    const DEFAULT_PER_PAGE: u64 = 50;

    pub fn new(
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        per_page: Option<u64>,
        cursor: Option<KeysetCursor<MediumId>>,
        tags: Vec<String>,
        album_id: Option<Uuid>,
        direction: Option<SortDirection>,
        include_no_album: bool,
    ) -> DomainResult<Self> {
        let per_page = per_page.unwrap_or(Self::DEFAULT_PER_PAGE);

        // Validate per_page is within bounds
        ensure!(
            per_page >= Self::MIN_PER_PAGE,
            ValidationSnafu {
                message: format!(
                    "per_page must be at least {}, got {}",
                    Self::MIN_PER_PAGE,
                    per_page
                ),
            }
        );

        ensure!(
            per_page <= Self::MAX_PER_PAGE,
            ValidationSnafu {
                message: format!(
                    "per_page cannot exceed {}, got {}",
                    Self::MAX_PER_PAGE,
                    per_page
                ),
            }
        );

        // Validate date range if both are present
        if let (Some(start), Some(end)) = (start_date, end_date) {
            ensure!(
                start <= end,
                ValidationSnafu {
                    message: "start_date must be before or equal to end_date",
                }
            );
        }

        Ok(Self {
            start_date,
            end_date,
            per_page,
            cursor,
            tags,
            album_id,
            direction: direction.unwrap_or_default(),
            include_no_album,
        })
    }

    /// Create a default filter with no criteria
    pub fn default_filter() -> Self {
        Self {
            start_date: None,
            end_date: None,
            per_page: Self::DEFAULT_PER_PAGE,
            cursor: None,
            tags: vec![],
            album_id: None,
            direction: SortDirection::default(),
            include_no_album: false,
        }
    }
}
