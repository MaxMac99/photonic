use std::env;

use mongodb::{Client, Collection};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, Credential};

use crate::entities::{Album, Medium};

mod album;
mod medium;

#[derive(Debug)]
pub struct Repository {
    medium_col: Collection<Medium>,
    album_col: Collection<Album>,
}

impl Repository {
    pub async fn init() -> Self {
        let mut opts = ClientOptions::parse(
            env::var("MONGO_URL").expect("No url provided"),
        )
            .await
            .expect("Invalid options for MongoDB");
        opts.credential = Some(
            Credential::builder()
                .username(env::var("MONGO_USER").expect("No user provided"))
                .password(env::var("MONGO_PASSWORD").expect("No password provided"))
                .build(),
        );

        let client =
            Client::with_options(opts).expect("Could not connect with MongoDB");
        let db = client.database("fotonic");
        db.run_command(doc! {"ping": 1}, None)
            .await
            .expect("Could not ping database");
        let medium_col: Collection<Medium> = db.collection("medium");
        let album_col: Collection<Album> = db.collection("album");

        Self { medium_col, album_col }
    }
}
