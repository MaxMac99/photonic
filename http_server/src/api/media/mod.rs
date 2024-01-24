use serde::{Deserialize, Serialize};

pub use create_medium::create_medium;
pub use find_all::find_all;
pub use raw::{
    get_medium_edit_raw, get_medium_original_raw, get_medium_preview_raw,
};

mod create_medium;
mod find_all;
mod model;
mod raw;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub next: Option<String>,
}
