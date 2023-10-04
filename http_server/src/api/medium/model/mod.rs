use serde::{Deserialize, Serialize};

pub mod input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medium {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub next: Option<String>,
}
