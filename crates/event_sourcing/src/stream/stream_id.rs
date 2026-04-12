use crate::aggregate::traits::AggregateType;

/// A typed stream identifier. Combines an aggregate type (category) with
/// a specific aggregate instance id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamId {
    aggregate_type: AggregateType,
    id: String,
}

impl StreamId {
    pub fn new(aggregate_type: AggregateType, id: impl Into<String>) -> Self {
        Self {
            aggregate_type,
            id: id.into(),
        }
    }

    pub fn aggregate_type(&self) -> &AggregateType {
        &self.aggregate_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the string representation used for storage (e.g. "Medium-uuid").
    pub fn to_storage_key(&self) -> String {
        format!("{}-{}", self.aggregate_type.name(), self.id)
    }
}
