use std::sync::Arc;

use mongodb::{Client, Collection};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, Credential};

use crate::{Config, Error};
use crate::entities::{Album, Medium};

mod album;
mod medium;

#[derive(Debug)]
pub struct Repository {
    medium_col: Collection<Medium>,
    album_col: Collection<Album>,
}

impl Repository {
    pub async fn init(config: Arc<Config>) -> Result<Self, Error> {
        let mut opts = ClientOptions::parse(config.mongo.url.clone())
            .await
            .map_err(|err| Error::Internal(format!("Invalid options for MongoDB: {}", err.to_string())))?;
        opts.credential = Some(
            Credential::builder()
                .username(config.mongo.username.clone())
                .password(config.mongo.password.clone())
                .build(),
        );

        let client = Client::with_options(opts)
            .map_err(|err| Error::Internal(format!("Could not connect to MongoDB: {}", err.to_string())))?;
        let db = client.database("fotonic");
        db.run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|err| Error::Internal(format!("Could not ping MongoDB: {}", err.to_string())))?;
        let medium_col: Collection<Medium> = db.collection("medium");
        let album_col: Collection<Album> = db.collection("album");

        Ok(Self {
            medium_col,
            album_col,
        })
    }
}
