use snafu::ensure;

use crate::error::{DomainResult, ValidationSnafu};

/// Camera information value object (denormalized from Metadata domain)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraInfo {
    make: String,
    model: String,
}

impl CameraInfo {
    const MAX_MAKE_LENGTH: usize = 100;
    const MAX_MODEL_LENGTH: usize = 100;

    pub fn new(make: impl Into<String>, model: impl Into<String>) -> DomainResult<Self> {
        let make = make.into();
        let model = model.into();

        ensure!(
            make.len() <= Self::MAX_MAKE_LENGTH,
            ValidationSnafu {
                message: format!(
                    "Camera make cannot exceed {} characters",
                    Self::MAX_MAKE_LENGTH
                ),
            }
        );

        ensure!(
            model.len() <= Self::MAX_MODEL_LENGTH,
            ValidationSnafu {
                message: format!(
                    "Camera model cannot exceed {} characters",
                    Self::MAX_MODEL_LENGTH
                ),
            }
        );

        Ok(Self { make, model })
    }

    pub fn make(&self) -> &str {
        &self.make
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get full camera name (make + model)
    pub fn full_name(&self) -> String {
        format!("{} {}", self.make, self.model)
    }
}

impl std::fmt::Display for CameraInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

/// GPS coordinates value object (denormalized from Metadata domain)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsCoordinates {
    latitude: f64,
    longitude: f64,
    altitude: Option<f64>,
}

impl GpsCoordinates {
    const MIN_LATITUDE: f64 = -90.0;
    const MAX_LATITUDE: f64 = 90.0;
    const MIN_LONGITUDE: f64 = -180.0;
    const MAX_LONGITUDE: f64 = 180.0;

    pub fn new(latitude: f64, longitude: f64, altitude: Option<f64>) -> DomainResult<Self> {
        ensure!(
            latitude >= Self::MIN_LATITUDE && latitude <= Self::MAX_LATITUDE,
            ValidationSnafu {
                message: format!(
                    "Latitude must be between {} and {}, got {}",
                    Self::MIN_LATITUDE,
                    Self::MAX_LATITUDE,
                    latitude
                ),
            }
        );

        ensure!(
            longitude >= Self::MIN_LONGITUDE && longitude <= Self::MAX_LONGITUDE,
            ValidationSnafu {
                message: format!(
                    "Longitude must be between {} and {}, got {}",
                    Self::MIN_LONGITUDE,
                    Self::MAX_LONGITUDE,
                    longitude
                ),
            }
        );

        Ok(Self {
            latitude,
            longitude,
            altitude,
        })
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn altitude(&self) -> Option<f64> {
        self.altitude
    }
}

impl std::fmt::Display for GpsCoordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.altitude {
            Some(alt) => write!(f, "{}, {} ({}m)", self.latitude, self.longitude, alt),
            None => write!(f, "{}, {}", self.latitude, self.longitude),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_info_valid() {
        let camera = CameraInfo::new("Canon", "EOS R5").unwrap();
        assert_eq!(camera.make(), "Canon");
        assert_eq!(camera.model(), "EOS R5");
        assert_eq!(camera.full_name(), "Canon EOS R5");
    }

    #[test]
    fn test_camera_info_too_long_fails() {
        let long_make = "a".repeat(101);
        let result = CameraInfo::new(long_make, "Model");
        assert!(result.is_err());
    }

    #[test]
    fn test_gps_coordinates_valid() {
        let gps = GpsCoordinates::new(37.7749, -122.4194, Some(10.0)).unwrap();
        assert_eq!(gps.latitude(), 37.7749);
        assert_eq!(gps.longitude(), -122.4194);
        assert_eq!(gps.altitude(), Some(10.0));
    }

    #[test]
    fn test_gps_coordinates_no_altitude() {
        let gps = GpsCoordinates::new(37.7749, -122.4194, None).unwrap();
        assert_eq!(gps.altitude(), None);
    }

    #[test]
    fn test_gps_coordinates_invalid_latitude() {
        let result = GpsCoordinates::new(91.0, 0.0, None);
        assert!(result.is_err());

        let result = GpsCoordinates::new(-91.0, 0.0, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_gps_coordinates_invalid_longitude() {
        let result = GpsCoordinates::new(0.0, 181.0, None);
        assert!(result.is_err());

        let result = GpsCoordinates::new(0.0, -181.0, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_gps_coordinates_boundary_values() {
        // Valid boundary values
        let gps = GpsCoordinates::new(90.0, 180.0, None).unwrap();
        assert_eq!(gps.latitude(), 90.0);
        assert_eq!(gps.longitude(), 180.0);

        let gps = GpsCoordinates::new(-90.0, -180.0, None).unwrap();
        assert_eq!(gps.latitude(), -90.0);
        assert_eq!(gps.longitude(), -180.0);
    }
}
