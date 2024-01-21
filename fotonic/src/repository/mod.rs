use std::sync::Arc;

use mongodb::{
    bson::doc,
    options::{ClientOptions, Credential},
    Client, Collection,
};
use snafu::{ResultExt, Whatever};

pub(crate) use medium::SaveMediumError;

use crate::{
    config::Config,
    entities::{Album, Medium},
};

mod album;
mod medium;

#[derive(Debug)]
pub struct Repository {
    medium_col: Collection<Medium>,
    album_col: Collection<Album>,
}

impl Repository {
    pub async fn init(config: Arc<Config>) -> Result<Self, Whatever> {
        let mut opts = ClientOptions::parse(config.mongo.url.clone())
            .await
            .with_whatever_context(|_| {
                format!("Mongo url error with {}", config.mongo.url)
            })?;
        opts.credential = Some(
            Credential::builder()
                .username(config.mongo.username.clone())
                .password(config.mongo.password.clone())
                .build(),
        );

        let client = Client::with_options(opts)
            .whatever_context("Mongo options error")?;
        let db = client.database("fotonic");
        db.run_command(doc! {"ping": 1}, None)
            .await
            .whatever_context("Could not ping mongo DB")?;
        let medium_col: Collection<Medium> = db.collection("medium");
        let album_col: Collection<Album> = db.collection("album");

        Ok(Self {
            medium_col,
            album_col,
        })
    }
}
