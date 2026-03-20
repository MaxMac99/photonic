use chrono::{DateTime, Utc};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UpdateOptional<T> {
    Ignore,
    Clear,
    SetIfNull(T),
    Replace(T),
}

impl<T> UpdateOptional<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> UpdateOptional<U> {
        match self {
            UpdateOptional::Ignore => UpdateOptional::Ignore,
            UpdateOptional::Clear => UpdateOptional::Clear,
            UpdateOptional::SetIfNull(value) => UpdateOptional::SetIfNull(f(value)),
            UpdateOptional::Replace(value) => UpdateOptional::Replace(f(value)),
        }
    }
}

/// Sort direction for query results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Descending
    }
}

/// Keyset cursor for efficient pagination
/// Uses composite key (date, id) for stable ordering
/// Generic over ID type to support different entity types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeysetCursor<ID> {
    pub last_date: DateTime<Utc>,
    pub last_id: ID,
}

impl<ID> KeysetCursor<ID> {
    pub fn new(last_date: DateTime<Utc>, last_id: ID) -> Self {
        Self { last_date, last_id }
    }
}
