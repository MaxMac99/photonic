use mongodb::Collection;

use crate::models::Medium;

mod create_medium;
pub mod repo;

pub struct MongoRepo {
    col: Collection<Medium>,
}