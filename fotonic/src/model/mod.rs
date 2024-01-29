use serde::Deserialize;

pub use album::Album;
pub use medium::{
    FileItem, Medium, MediumItem, MediumItemType, MediumType, StoreLocation,
};
pub use trash::TrashItem;

mod album;
mod medium;
mod trash;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum DateDirection {
    #[serde(rename = "DESC")]
    NewestFirst,
    #[serde(rename = "ASC")]
    OldestFirst,
}
