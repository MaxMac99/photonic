use std::{
    marker::PhantomData,
    mem::replace,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project_lite::pin_project;

pub trait GroupedRow<A, K> {
    fn key(&self) -> &K;

    fn start_parent(self) -> A;

    fn push_into(self, parent: &mut A);
}

pin_project! {
    pub struct GroupedStream<S, R, A, K> {
        #[pin]
        inner: S,
        current: Option<(K, A)>,
        row: PhantomData<R>,
    }
}

impl<S, R, A, K> GroupedStream<S, R, A, K> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            current: None,
            row: PhantomData,
        }
    }
}

impl<S, R, A, K> Stream for GroupedStream<S, R, A, K>
where
    S: Stream<Item = Result<R, sqlx::Error>>,
    R: GroupedRow<A, K>,
    K: Eq + Clone,
{
    type Item = Result<A, sqlx::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(row))) => {
                    let row_key = row.key().clone();
                    match this.current {
                        Some((current_key, parent)) if &row_key == current_key => {
                            row.push_into(parent);
                        }
                        Some((current_key, parent)) => {
                            let completed = replace(parent, row.start_parent());
                            *current_key = row_key;
                            return Poll::Ready(Some(Ok(completed)));
                        }
                        None => {
                            *this.current = Some((row_key, row.start_parent()));
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    return Poll::Ready(this.current.take().map(|(_, parent)| Ok(parent)))
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

pub trait GroupedStreamExt<R, A>: Stream<Item = Result<R, sqlx::Error>> {
    fn grouped<K>(self) -> GroupedStream<Self, R, A, K>
    where
        Self: Sized,
        R: GroupedRow<A, K>,
        K: Eq;
}

impl<S, R, A> GroupedStreamExt<R, A> for S
where
    S: Stream<Item = Result<R, sqlx::Error>>,
{
    fn grouped<K>(self) -> GroupedStream<Self, R, A, K>
    where
        Self: Sized,
        R: GroupedRow<A, K>,
        K: Eq,
    {
        GroupedStream::new(self)
    }
}
