use serde::Deserialize;

pub use access::Access;
pub use album::Album;
pub use medium::{FileItem, Medium, MediumItem, MediumItemType, MediumType, StoreLocation};
pub use trash::TrashItem;
pub use user::User;

mod access;
mod album;
mod medium;
mod trash;
mod user;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum DateDirection {
    #[serde(rename = "DESC")]
    NewestFirst,
    #[serde(rename = "ASC")]
    OldestFirst,
}
