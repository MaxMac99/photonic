pub mod event_store;
pub mod repository;
pub mod traits;

pub use repository::AggregateRepository;
pub use traits::{Aggregate, AggregateType, ApplyEvent};
