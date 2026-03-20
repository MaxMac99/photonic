use std::time::Duration;

use tokio::time::{sleep, timeout, Instant};

/// Backoff strategy for polling
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed interval between retries
    Fixed,
    /// Exponential backoff with multiplier
    Exponential { factor: f64, max_interval: Duration },
}

/// Configuration for polling operations in tests
#[derive(Debug, Clone)]
pub struct PollingConfig {
    pub timeout: Duration,
    pub interval: Duration,
    pub description: String,
    pub backoff: BackoffStrategy,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            interval: Duration::from_millis(100),
            description: "operation".to_string(),
            backoff: BackoffStrategy::Fixed,
        }
    }
}

impl PollingConfig {
    /// Create a new polling config with a description
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            ..Default::default()
        }
    }

    /// Set the timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the polling interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Use exponential backoff strategy
    ///
    /// # Arguments
    /// * `factor` - Multiplier for each retry (e.g., 2.0 doubles the interval each time)
    /// * `max_interval` - Maximum interval to wait between retries
    pub fn with_exponential_backoff(mut self, factor: f64, max_interval: Duration) -> Self {
        self.backoff = BackoffStrategy::Exponential {
            factor,
            max_interval,
        };
        self
    }

    /// Create a config for quick operations (2s timeout, 50ms interval)
    pub fn quick(description: impl Into<String>) -> Self {
        Self {
            timeout: Duration::from_secs(2),
            interval: Duration::from_millis(50),
            description: description.into(),
            backoff: BackoffStrategy::Fixed,
        }
    }

    /// Create a config for long operations (30s timeout, 200ms interval)
    pub fn long(description: impl Into<String>) -> Self {
        Self {
            timeout: Duration::from_secs(30),
            interval: Duration::from_millis(200),
            description: description.into(),
            backoff: BackoffStrategy::Fixed,
        }
    }
}

/// Poll a condition until it returns Some(T) or times out
///
/// This function repeatedly calls the `check` closure until it returns `Some(value)`,
/// or the timeout is reached. It provides detailed error messages for test failures.
///
/// # Example
///
/// ```rust
/// let result = poll_until(
///     || async {
///         let response = make_request().await.ok()?;
///         let data = response.json().await.ok()?;
///         data.ready.then_some(data)
///     },
///     PollingConfig::new("data to be ready"),
/// )
/// .await
/// .expect("Data was not ready in time");
/// ```
pub async fn poll_until<F, Fut, T>(mut check: F, config: PollingConfig) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let start = Instant::now();
    let mut attempts = 0;
    let mut current_interval = config.interval;

    let result = timeout(config.timeout, async {
        loop {
            attempts += 1;

            if let Some(result) = check().await {
                return Ok::<T, String>(result);
            }

            sleep(current_interval).await;

            // Calculate next interval based on backoff strategy
            match &config.backoff {
                BackoffStrategy::Fixed => {
                    // Keep current_interval unchanged
                }
                BackoffStrategy::Exponential {
                    factor,
                    max_interval,
                } => {
                    current_interval = Duration::from_secs_f64(
                        (current_interval.as_secs_f64() * factor).min(max_interval.as_secs_f64()),
                    );
                }
            }
        }
    })
    .await;

    match result {
        Ok(Ok(value)) => {
            tracing::debug!(
                "✓ {} completed after {} attempts in {:?}",
                config.description,
                attempts,
                start.elapsed()
            );
            Ok(value)
        }
        _ => Err(format!(
            "✗ {} timed out after {} attempts ({:?}). Expected within {:?}, starting with {:?} interval",
            config.description,
            attempts,
            start.elapsed(),
            config.timeout,
            config.interval
        )),
    }
}

/// Poll until a condition returns true
///
/// # Example
///
/// ```rust
/// poll_until_true(
///     || async { is_ready().await },
///     PollingConfig::new("system to be ready"),
/// )
/// .await
/// .expect("System was not ready in time");
/// ```
pub async fn poll_until_true<F, Fut>(check: F, config: PollingConfig) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    poll_until(
        || async {
            if check().await {
                Some(())
            } else {
                None
            }
        },
        config,
    )
    .await
}

/// Poll with Result-based check function
///
/// Continues polling on Err, returns on Ok.
///
/// # Example
///
/// ```rust
/// let data = poll_with_result(
///     || async {
///         let response = make_request().await?;
///         if response.is_ready {
///             Ok(response.data)
///         } else {
///             Err("not ready")
///         }
///     },
///     PollingConfig::new("data to be ready"),
/// )
/// .await
/// .expect("Data was not ready in time");
/// ```
pub async fn poll_with_result<F, Fut, T, E>(check: F, config: PollingConfig) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    poll_until(
        || async {
            match check().await {
                Ok(result) => Some(result),
                Err(_) => None,
            }
        },
        config,
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    use super::*;

    #[tokio::test]
    async fn test_poll_until_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = poll_until(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if count >= 3 {
                        Some(42)
                    } else {
                        None
                    }
                }
            },
            PollingConfig::new("counter to reach 3").with_interval(Duration::from_millis(10)),
        )
        .await;

        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_poll_until_timeout() {
        let result = poll_until(
            || async { None::<i32> },
            PollingConfig::new("impossible condition")
                .with_timeout(Duration::from_millis(100))
                .with_interval(Duration::from_millis(10)),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[tokio::test]
    async fn test_poll_until_true_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = poll_until_true(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    count >= 3
                }
            },
            PollingConfig::new("counter check").with_interval(Duration::from_millis(10)),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_poll_with_result_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = poll_with_result(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if count >= 3 {
                        Ok::<_, String>(42)
                    } else {
                        Err("not ready".to_string())
                    }
                }
            },
            PollingConfig::new("result check").with_interval(Duration::from_millis(10)),
        )
        .await;

        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        use tokio::time::Instant;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let start = Instant::now();

        let result = poll_until(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if count >= 3 {
                        Some(42)
                    } else {
                        None
                    }
                }
            },
            PollingConfig::new("exponential backoff test")
                .with_interval(Duration::from_millis(10))
                .with_exponential_backoff(2.0, Duration::from_millis(100)),
        )
        .await;

        let elapsed = start.elapsed();

        // With exponential backoff: 10ms + 20ms = 30ms minimum
        // Without backoff: 10ms + 10ms = 20ms
        // So elapsed should be >= 30ms
        assert_eq!(result, Ok(42));
        assert!(
            elapsed >= Duration::from_millis(25),
            "Expected at least 25ms with exponential backoff, got {:?}",
            elapsed
        );
    }
}
