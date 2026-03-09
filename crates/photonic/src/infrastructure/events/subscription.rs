/// Options for configuring event subscription behavior
#[derive(Clone)]
pub struct SubscriptionOptions {
    /// Number of worker tasks in this consumer group
    /// Each worker processes events sequentially, one at a time
    /// Multiple workers enable parallel task with load balancing
    /// Default is 1
    pub workers: usize,
    /// Optional name for debugging/metrics
    pub name: Option<String>,
}

impl SubscriptionOptions {
    /// Create options with default settings (1 worker)
    pub fn new() -> Self {
        Self {
            workers: 1,
            name: None,
        }
    }

    /// Set the number of worker tasks for parallel task
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers.max(1); // Ensure at least 1 worker
        self
    }

    /// Builder method to set name
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self::new()
    }
}
