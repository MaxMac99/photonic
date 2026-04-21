/// Where to start consuming events from.
#[derive(Clone, Default)]
pub enum StartFrom {
    /// Start from the beginning of the stream. No offset tracking.
    /// Replays all events on every startup.
    Beginning,
    /// Start from the latest event. No offset tracking.
    /// Only receives events published after subscribing.
    #[default]
    Latest,
    /// Resume from last committed checkpoint. On first start, begins
    /// from the beginning. The consumer name is used as the checkpoint key.
    Checkpoint { consumer_name: String },
}

/// Options for configuring event subscription behavior.
#[derive(Clone)]
pub struct SubscriptionOptions {
    /// Number of worker tasks in this consumer group.
    /// Each worker processes events sequentially, one at a time.
    /// Multiple workers enable parallel processing with load balancing.
    /// Default is 1.
    pub workers: usize,
    /// Optional name for debugging/metrics. Separate from the checkpoint
    /// consumer name — this is only used for logging and task naming.
    pub name: Option<String>,
    /// Where to start consuming from. Default is `Latest`.
    pub start_from: StartFrom,
}

impl SubscriptionOptions {
    /// Create options with default settings (1 worker, start from latest).
    pub fn new() -> Self {
        Self {
            workers: 1,
            name: None,
            start_from: StartFrom::default(),
        }
    }

    /// Set the number of worker tasks for parallel processing.
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers.max(1);
        self
    }

    /// Set a name for debugging/metrics (task naming, logging).
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set where to start consuming from.
    pub fn start_from(mut self, start_from: StartFrom) -> Self {
        self.start_from = start_from;
        self
    }
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self::new()
    }
}
