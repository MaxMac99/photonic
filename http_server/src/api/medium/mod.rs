use serde::{Deserialize, Serialize};

pub use create_medium::create_medium;
pub use find_all::find_all;

mod create_medium;
mod find_all;
mod model;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub next: Option<String>,
}
