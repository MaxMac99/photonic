// Custom assertion helpers for tests

#![allow(dead_code)]

use photonic::domain::error::DomainError;
use uuid::Uuid;

/// Assert that an error is an EntityNotFound error with specific entity and ID
pub fn assert_not_found_error<T>(
    result: Result<T, DomainError>,
    expected_entity: &str,
    expected_id: Uuid,
) {
    match result {
        Err(DomainError::EntityNotFound {
            entity,
            id,
            backtrace,
        }) => {
            assert_eq!(entity, expected_entity, "Entity type mismatch");
            assert_eq!(id, expected_id, "ID mismatch");
        }
        Ok(_) => panic!("Expected EntityNotFound error, got Ok"),
        Err(e) => panic!("Expected EntityNotFound error, got {:?}", e),
    }
}

/// Assert that an error is a Validation error containing specific message
pub fn assert_validation_error<T>(result: Result<T, DomainError>, expected_message_fragment: &str) {
    match result {
        Err(DomainError::Validation { message, .. }) => {
            assert!(
                message.contains(expected_message_fragment),
                "Expected message to contain '{}', got '{}'",
                expected_message_fragment,
                message
            );
        }
        Ok(_) => panic!("Expected Validation error, got Ok"),
        Err(e) => panic!("Expected Validation error, got {:?}", e),
    }
}
