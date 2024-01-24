use serde::Deserialize;

pub use album::Album;
pub use medium::{FileItem, Medium, MediumItem, MediumType, StoreLocation};

mod album;
mod medium;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum DateDirection {
    #[serde(rename = "DESC")]
    NewestFirst,
    #[serde(rename = "ASC")]
    OldestFirst,
}
