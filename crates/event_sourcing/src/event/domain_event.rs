use crate::event::event_metadata::EventMetadata;
use std::any::Any;
use std::fmt::Debug;

pub trait DomainEvent: Any + Send + Sync + Debug {
    fn metadata(&self) -> &EventMetadata;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct TestEvent {
        pub metadata: EventMetadata,
        pub value: String,
    }

    impl TestEvent {
        pub fn new(value: &str) -> Self {
            Self {
                metadata: EventMetadata::default(),
                value: value.to_string(),
            }
        }
    }

    impl DomainEvent for TestEvent {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct OtherEvent {
        pub metadata: EventMetadata,
    }

    impl DomainEvent for OtherEvent {
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
    }
}
