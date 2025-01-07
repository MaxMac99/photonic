use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, utoipa::ToSchema, PartialEq)]
pub enum Direction {
    Asc,
    Desc,
}

impl Direction {
    pub(crate) fn to_sql(&self) -> &'static str {
        match self {
            Direction::Asc => "ASC",
            Direction::Desc => "DESC",
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Desc
    }
}
