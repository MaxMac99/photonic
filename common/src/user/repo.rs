use crate::{error::Result, ksqldb::KsqlDb};
use byte_unit::Byte;
use futures_util::StreamExt;
use serde::Deserialize;
use snafu::{ResultExt, Whatever};
use std::sync::Arc;
use tracing::log::{debug, error};
use uuid::Uuid;

pub async fn setup_streams_and_tables(ksql_db: Arc<KsqlDb>) -> std::result::Result<(), Whatever> {
    ksql_db
        .create(
            r#"
    CREATE SOURCE STREAM IF NOT EXISTS user_events (
        user VARCHAR,
        "size" BIGINT
    ) WITH (
        KAFKA_TOPIC = 'MediumItemCreated',
        VALUE_FORMAT = 'AVRO'
    );"#,
            &Default::default(),
            None,
        )
        .await
        .whatever_context("Could not create \"user_events\" stream")?;
    ksql_db
        .create(
            r#"
    CREATE TABLE IF NOT EXISTS users
    AS SELECT
        user as user_id,
        SUM("size") as quota_used
    FROM user_events
    GROUP BY user;"#,
            &Default::default(),
            None,
        )
        .await
        .whatever_context("Could not create \"users\" table")?;
    Ok(())
}

pub async fn get_current_quota_usage(ksql_db: Arc<KsqlDb>, user_id: Uuid) -> Result<Byte> {
    #[derive(Debug, Deserialize)]
    struct QuotaResponse {
        quota_used: u64,
    }

    let mut stream = ksql_db
        .query::<QuotaResponse>(
            &format!(
                "SELECT quota_used FROM users WHERE user_id = '{}';",
                user_id
            ),
            &Default::default(),
        )
        .await?;
    let usage = stream
        .next()
        .await
        .unwrap_or_else(|| {
            debug!("KSQL was empty");
            Ok(QuotaResponse { quota_used: 0 })
        })
        .unwrap_or_else(|err| {
            error!("KSQL Error: {}", err);
            QuotaResponse { quota_used: 0 }
        });

    Ok(Byte::from_u64(usage.quota_used))
}
