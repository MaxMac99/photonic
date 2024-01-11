use serde::{Deserialize, Serialize};

pub(crate) mod create_medium;
pub(crate) mod find_all;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub next: Option<String>,
}
