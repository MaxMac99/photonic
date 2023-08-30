use mongodb::Collection;

use crate::models::{Album, Medium};

mod medium;
pub mod repo;
mod album;

pub struct MongoRepo {
    medium_col: Collection<Medium>,
    album_col: Collection<Album>,
}