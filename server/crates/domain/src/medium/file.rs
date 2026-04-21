use std::path::Path;

use serde::{Deserialize, Serialize};
use snafu::ensure;

use crate::error::{DomainResult, ValidationSnafu};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimensions {
    width: u32,
    height: u32,
}

impl Dimensions {
    const MIN_DIMENSION: u32 = 1;
    const MAX_DIMENSION: u32 = 100_000;

    pub fn new(width: u32, height: u32) -> DomainResult<Self> {
        ensure!(
            width >= Self::MIN_DIMENSION,
            ValidationSnafu {
                message: format!(
                    "Width must be at least {}, got {}",
                    Self::MIN_DIMENSION,
                    width
                ),
            }
        );

        ensure!(
            height >= Self::MIN_DIMENSION,
            ValidationSnafu {
                message: format!("Height must be at least {}", Self::MIN_DIMENSION),
            }
        );

        ensure!(
            width <= Self::MAX_DIMENSION,
            ValidationSnafu {
                message: format!("Width cannot exceed {}, got {}", Self::MAX_DIMENSION, width),
            }
        );

        ensure!(
            height <= Self::MAX_DIMENSION,
            ValidationSnafu {
                message: format!(
                    "Height cannot exceed {}, got {}",
                    Self::MAX_DIMENSION,
                    height
                ),
            }
        );

        Ok(Self { width, height })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn total_pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    pub fn is_portrait(&self) -> bool {
        self.height > self.width
    }

    pub fn is_square(&self) -> bool {
        self.width == self.height
    }
}

impl std::fmt::Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filename(String);

impl Filename {
    const MAX_LENGTH: usize = 255;

    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();

        ensure!(
            !value.trim().is_empty(),
            ValidationSnafu {
                message: "Filename cannot be empty",
            }
        );

        ensure!(
            value.len() <= Self::MAX_LENGTH,
            ValidationSnafu {
                message: format!(
                    "Filename cannot exceed {} characters, got {}",
                    Self::MAX_LENGTH,
                    value.len()
                ),
            }
        );

        ensure!(
            !value.contains('\0'),
            ValidationSnafu {
                message: "Filename cannot contain null bytes",
            }
        );

        ensure!(
            !value.contains(".."),
            ValidationSnafu {
                message: "Filename cannot contain path traversal sequences",
            }
        );

        ensure!(
            !value.starts_with('/') && !value.starts_with('\\'),
            ValidationSnafu {
                message: "Filename cannot be an absolute path",
            }
        );

        let path = Path::new(&value);
        ensure!(
            path.file_stem().is_some(),
            ValidationSnafu {
                message: "Filename must have a name (stem)",
            }
        );

        ensure!(
            path.extension().is_some(),
            ValidationSnafu {
                message: "Filename must have an extension",
            }
        );

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get file extension if present
    pub fn extension(&self) -> &str {
        Path::new(&self.0)
            .extension()
            .and_then(|ext| ext.to_str())
            .expect("Extension")
    }

    /// Get filename without extension
    pub fn stem(&self) -> &str {
        Path::new(&self.0)
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Filename")
    }
}

impl AsRef<str> for Filename {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Filename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Priority value object for MediumItem ordering
/// Lower numbers = higher priority (e.g., 0 is highest priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Priority(i32);

impl Priority {
    pub const HIGHEST: i32 = 0;
    pub const HIGH: i32 = 10;
    pub const NORMAL: i32 = 50;
    pub const LOW: i32 = 100;

    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn highest() -> Self {
        Self(Self::HIGHEST)
    }

    pub fn high() -> Self {
        Self(Self::HIGH)
    }

    pub fn normal() -> Self {
        Self(Self::NORMAL)
    }

    pub fn low() -> Self {
        Self(Self::LOW)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    /// Check if this priority is higher than another
    /// (lower number = higher priority)
    pub fn is_higher_than(&self, other: &Priority) -> bool {
        self.0 < other.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::normal()
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensions_valid() {
        let dims = Dimensions::new(1920, 1080).unwrap();
        assert_eq!(dims.width(), 1920);
        assert_eq!(dims.height(), 1080);
        assert!(dims.is_landscape());
        assert!(!dims.is_portrait());
    }

    #[test]
    fn test_dimensions_zero_width_fails() {
        let result = Dimensions::new(0, 1080);
        assert!(result.is_err());
    }

    #[test]
    fn test_dimensions_too_large_fails() {
        let result = Dimensions::new(100_001, 1080);
        assert!(result.is_err());
    }

    #[test]
    fn test_dimensions_aspect_ratio() {
        let dims = Dimensions::new(1920, 1080).unwrap();
        assert!((dims.aspect_ratio() - 1.777).abs() < 0.01);
    }

    #[test]
    fn test_dimensions_orientation() {
        let landscape = Dimensions::new(1920, 1080).unwrap();
        assert!(landscape.is_landscape());

        let portrait = Dimensions::new(1080, 1920).unwrap();
        assert!(portrait.is_portrait());

        let square = Dimensions::new(1080, 1080).unwrap();
        assert!(square.is_square());
    }

    #[test]
    fn test_filename_valid() {
        let filename = Filename::new("image.jpg").unwrap();
        assert_eq!(filename.as_str(), "image.jpg");
        assert_eq!(filename.extension(), "jpg");
        assert_eq!(filename.stem(), "image");
    }

    #[test]
    fn test_filename_empty_fails() {
        let result = Filename::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_filename_path_traversal_fails() {
        let result = Filename::new("../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_filename_absolute_path_fails() {
        let result = Filename::new("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_filename_null_byte_fails() {
        let result = Filename::new("file\0.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_priority_ordering() {
        let p1 = Priority::highest();
        let p2 = Priority::normal();
        assert!(p1.is_higher_than(&p2));
        assert!(!p2.is_higher_than(&p1));
    }

    #[test]
    fn test_priority_constants() {
        assert_eq!(Priority::highest().value(), 0);
        assert_eq!(Priority::high().value(), 10);
        assert_eq!(Priority::normal().value(), 50);
        assert_eq!(Priority::low().value(), 100);
    }
}
