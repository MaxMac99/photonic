use std::env;
use std::path::PathBuf;

use tokio::fs;

use crate::repository::photo::PhotoRepo;

impl PhotoRepo {
    pub async fn init() -> Self {
        let base_path = PathBuf::from(
            env::var("PHOTO_BASE_PATH").unwrap_or("/photos".to_string()),
        );
        fs::create_dir_all(&base_path)
            .await
            .expect("Could not create base path");
        let canonicalized = fs::canonicalize(base_path)
            .await
            .expect("Could not create base path");

        let pattern = env::var("PHOTO_PATTERN")
            .unwrap_or("/<album_year>/<album>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>".to_string());

        PhotoRepo {
            base_path: canonicalized,
            pattern,
        }
    }
}
