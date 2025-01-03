use crate::util::stream::{JoinStream, Keyed};
use futures::Stream;
use std::hash::Hash;

impl<T: ?Sized> StreamExt for T where T: Stream {}

pub trait StreamExt: Stream {
    fn join<R, KL, KR, V>(self, other: R) -> JoinStream<Self, R, KL, KR, V>
    where
        V: Hash + Eq + Clone,
        KL: Keyed<V>,
        KR: Keyed<V>,
        R: Stream<Item = KR>,
        Self: Sized + Stream<Item = KL>,
    {
        JoinStream::new(self, other)
    }
}
