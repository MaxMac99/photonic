use std::sync::Arc;

use chrono::{Duration, Utc};
use tokio::time;
use tracing::{error, info};

use crate::application::medium::commands::{
    CleanupExpiredTempStorageCommand, CleanupExpiredTempStorageHandler,
};

pub fn spawn_cleanup_task(
    handler: Arc<CleanupExpiredTempStorageHandler>,
    temp_ttl_seconds: u64,
    cleanup_interval_seconds: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval_duration =
            std::time::Duration::from_secs(cleanup_interval_seconds);
        let mut interval = time::interval(interval_duration);

        // Skip the first immediate tick
        interval.tick().await;

        info!(
            interval_seconds = cleanup_interval_seconds,
            ttl_seconds = temp_ttl_seconds,
            "Temp storage cleanup task started"
        );

        loop {
            interval.tick().await;

            let cutoff = Utc::now()
                - Duration::seconds(temp_ttl_seconds as i64);

            if let Err(e) = handler
                .handle(CleanupExpiredTempStorageCommand { cutoff })
                .await
            {
                error!(error = %e, "Temp storage cleanup sweep encountered an error");
            }
        }
    })
}