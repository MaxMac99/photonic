use futures::Stream;
use futures_util::stream::{Fuse, FusedStream};
use pin_project_lite::pin_project;
use std::{
    collections::HashMap,
    hash::Hash,
    pin::Pin,
    task::{Context, Poll},
};

pub trait Keyed<K> {
    fn get_key(&self) -> K;
}

pin_project! {
    pub struct JoinStream<L, R, KL, KR, V>
    where
        KL: Keyed<V>,
        KR: Keyed<V>,
        L: Stream<Item = KL>,
        R: Stream<Item = KR>,
        V: Hash,
    {
        #[pin]
        left: Fuse<L>,
        #[pin]
        right: Fuse<R>,
        events: HashMap<V, (Option<KL>, Option<KR>)>,
    }
}

impl<L, R, KL, KR, V> JoinStream<L, R, KL, KR, V>
where
    V: Hash + Eq + Clone,
    KL: Keyed<V>,
    KR: Keyed<V>,
    L: Stream<Item = KL>,
    R: Stream<Item = KR>,
{
}

impl<L, R, KL, KR, V> FusedStream for JoinStream<L, R, KL, KR, V>
where
    V: Hash + Eq + Clone,
    KL: Keyed<V>,
    KR: Keyed<V>,
    L: Stream<Item = KL>,
    R: Stream<Item = KR>,
{
    fn is_terminated(&self) -> bool {
        self.left.is_terminated() && self.right.is_terminated()
    }
}

impl<L, R, KL, KR, V> Stream for JoinStream<L, R, KL, KR, V>
where
    V: Hash + Eq + Clone,
    KL: Keyed<V>,
    KR: Keyed<V>,
    L: Stream<Item = KL>,
    R: Stream<Item = KR>,
{
    type Item = (KL, KR);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.left.as_mut().poll_next(cx) {
            Poll::Ready(Some(event)) => {
                if let Some(entry) = this.events.remove(&event.get_key()) {
                    let (_, right) = entry;
                    let right = right.expect("Left event missing");
                    return Poll::Ready(Some((event, right)));
                } else {
                    this.events
                        .insert(event.get_key().clone(), (Some(event), None));
                }
            }
            Poll::Ready(None) | Poll::Pending => {}
        }

        match this.right.as_mut().poll_next(cx) {
            Poll::Ready(Some(event)) => {
                if let Some(entry) = this.events.remove(&event.get_key()) {
                    let (left, _) = entry;
                    let left = left.expect("Right event missing");
                    return Poll::Ready(Some((left, event)));
                } else {
                    this.events
                        .insert(event.get_key().clone(), (None, Some(event)));
                }
            }
            Poll::Ready(None) | Poll::Pending => {}
        }

        if this.left.is_done() || this.right.is_done() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}
