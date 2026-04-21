use std::path::PathBuf;

use chrono::{Datelike, Utc};

use super::{Medium, MediumItem};

/// Domain service that determines where media files should be stored
/// based on configurable patterns using `<token>` syntax from StorageConfig.
///
/// Supported tokens:
/// - `<year>`: Year from taken_at (falls back to created_at)
/// - `<month>`: Two-digit month
/// - `<day>`: Two-digit day
/// - `<camera_make>`: Camera manufacturer
/// - `<camera_model>`: Camera model
/// - `<filename>`: Original filename without extension
/// - `<extension>`: File extension
/// - `<medium_type>`: Type of medium (photo, video, etc.)
/// - `<id>`: Medium ID
pub struct StoragePathService {
    pattern: String,
}

impl StoragePathService {
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }

    /// Generates the full relative file path for permanent storage
    /// based on the configured pattern and the medium/item metadata.
    /// Missing values are replaced with defaults.
    pub fn generate_permanent_path(&self, medium: &Medium, item: &MediumItem) -> PathBuf {
        let date = medium
            .taken_at
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or(medium.created_at);

        let path = self
            .pattern
            .replace("<year>", &date.year().to_string())
            .replace("<month>", &format!("{:02}", date.month()))
            .replace("<day>", &format!("{:02}", date.day()))
            .replace(
                "<camera_make>",
                medium.camera_make.as_deref().unwrap_or("unknown"),
            )
            .replace(
                "<camera_model>",
                medium.camera_model.as_deref().unwrap_or("unknown"),
            )
            .replace("<filename>", item.filename.stem())
            .replace("<extension>", item.filename.extension())
            .replace(
                "<medium_type>",
                &format!("{:?}", medium.medium_type).to_lowercase(),
            )
            .replace("<id>", &medium.id.to_string());

        Self::sanitize_path(PathBuf::from(path))
    }

    fn sanitize_path(path: PathBuf) -> PathBuf {
        path.components()
            .filter(|c| {
                !matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::RootDir
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use byte_unit::Byte;
    use chrono::DateTime;
    use uuid::Uuid;

    use super::*;
    use crate::medium::{
        file::{Filename, Priority},
        storage::FileLocation,
        MediumItemType, MediumType,
    };

    fn make_medium(
        taken_at: Option<DateTime<chrono::FixedOffset>>,
        camera_make: Option<&str>,
        camera_model: Option<&str>,
    ) -> Medium {
        Medium {
            id: Uuid::new_v4(),
            owner_id: Default::default(),
            medium_type: MediumType::Photo,
            leading_item_id: Default::default(),
            taken_at,
            camera_make: camera_make.map(String::from),
            camera_model: camera_model.map(String::from),
            gps_coordinates: None,
            created_at: DateTime::parse_from_rfc3339("2024-03-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            updated_at: Utc::now(),
            items: vec![],
            version: 0,
        }
    }

    fn make_item(filename: &str) -> MediumItem {
        MediumItem {
            id: Uuid::new_v4(),
            medium_id: Uuid::new_v4(),
            medium_item_type: MediumItemType::Original,
            mime: "image/heic".parse().unwrap(),
            filename: Filename::new(filename).unwrap(),
            filesize: Byte::from_u64(1000),
            priority: Priority::normal(),
            dimensions: None,
            locations: vec![FileLocation::temporary("test.heic".into())],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_generate_permanent_path_with_full_metadata() {
        let service = StoragePathService::new(
            "<year>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>".to_string(),
        );

        let taken_at = DateTime::parse_from_rfc3339("2024-06-20T14:30:00+02:00").ok();
        let medium = make_medium(taken_at, Some("Apple"), Some("iPhone 15 Pro"));
        let item = make_item("IMG_4598.HEIC");

        let path = service.generate_permanent_path(&medium, &item);
        let path_str = path.to_string_lossy();

        assert_eq!(path_str, "2024/0620/Apple_iPhone 15 Pro/IMG_4598.HEIC");
    }

    #[test]
    fn test_generate_permanent_path_with_defaults() {
        let service = StoragePathService::new(
            "<year>/<month><day>/<camera_make>_<camera_model>/<filename>.<extension>".to_string(),
        );

        let medium = make_medium(None, None, None);
        let item = make_item("photo.jpg");

        let path = service.generate_permanent_path(&medium, &item);
        let path_str = path.to_string_lossy();

        // Falls back to created_at (2024-03-15) and "unknown" for camera
        assert_eq!(path_str, "2024/0315/unknown_unknown/photo.jpg");
    }

    #[test]
    fn test_sanitize_removes_root_and_parent() {
        let service = StoragePathService::new("/<year>/../<filename>.<extension>".to_string());

        let medium = make_medium(None, None, None);
        let item = make_item("test.jpg");

        let path = service.generate_permanent_path(&medium, &item);
        assert!(!path.to_string_lossy().starts_with('/'));
        assert!(!path.to_string_lossy().contains(".."));
    }

    #[test]
    fn test_simple_pattern() {
        let service =
            StoragePathService::new("<medium_type>/<id>/<filename>.<extension>".to_string());

        let medium = make_medium(None, None, None);
        let item = make_item("IMG_4598.HEIC");

        let path = service.generate_permanent_path(&medium, &item);
        let path_str = path.to_string_lossy();

        assert!(path_str.starts_with("photo/"));
        assert!(path_str.ends_with("/IMG_4598.HEIC"));
    }
}
