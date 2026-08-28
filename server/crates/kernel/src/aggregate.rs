pub type AggregateVersion = i64;

/// Core event sourcing trait for aggregate roots.
/// Aggregates are reconstituted by replaying events through type-specific
/// `ApplyEvent<E>` implementations.
pub trait AggregateRoot: Send + Sync + Sized {
    /// Unique aggregate type name (used as stream prefix in event store)
    fn aggregate_type() -> &'static str;

    /// Current version (number of events applied)
    fn version(&self) -> AggregateVersion;
}
