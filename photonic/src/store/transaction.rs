use crate::error::Error;
use futures_util::future::BoxFuture;
use tokio::{runtime::Handle, task};

pub struct Transaction<'a> {
    open: bool,
    history: Vec<Box<dyn FnOnce() -> BoxFuture<'a, Result<(), Error>>>>,
}

impl<'a> Transaction<'a> {
    pub fn begin() -> Self {
        Self {
            open: true,
            history: vec![],
        }
    }

    pub(super) fn add_rollback<F>(&mut self, function: F)
    where
        F: FnOnce() -> BoxFuture<'a, Result<(), Error>> + 'static,
    {
        self.history.push(Box::new(function));
    }

    pub fn commit(mut self) {
        self.open = false;
    }

    pub async fn rollback(mut self) -> Result<(), Error> {
        self.rollback_internal().await
    }

    async fn rollback_internal(&mut self) -> Result<(), Error> {
        while let Some(item) = self.history.pop() {
            item().await?;
        }
        self.open = false;
        Ok(())
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.open {
            task::block_in_place(move || {
                Handle::current().block_on(async move {
                    self.rollback_internal().await.expect("TODO: panic message");
                });
            });
        }
    }
}
