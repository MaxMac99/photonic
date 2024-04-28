use serde::{Deserialize, Serialize};

pub use add_raw::add_raw;
pub use create_medium::create_medium;
pub use delete::delete_medium;
pub use find_all::find_all;
pub use get_raw::get_medium_item_raw;

mod add_raw;
mod create_medium;
mod delete;
mod find_all;
mod get_raw;
mod model;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub next: Option<String>,
}
