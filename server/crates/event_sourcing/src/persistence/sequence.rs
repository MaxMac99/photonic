/// Represents a position in the event stream.
///
/// Implementations must provide a starting value and the ability to
/// determine whether a new sequence supersedes the current one.
pub trait Sequence: Copy + Default + PartialEq + Send + Sync + 'static {
    /// Returns `true` if `other` represents a later position in the stream.
    fn is_behind(&self, other: &Self) -> bool;

    /// Returns `true` if `other` is the immediate next position after `self`.
    fn is_next(&self, other: &Self) -> bool;
}

impl Sequence for i64 {
    fn is_behind(&self, other: &i64) -> bool {
        *self < *other
    }

    fn is_next(&self, other: &i64) -> bool {
        *self + 1 == *other
    }
}
