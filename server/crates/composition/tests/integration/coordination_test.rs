use event_sourcing::persistence::checkpoint_store::TxCheckpointStore;
use postgres::persistence::postgres::checkpoint_store::PostgresCoordinatedTxCheckpointStore;
use serial_test::serial;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

// ============================================================================
// COORDINATION-SAFE CHECKPOINT STORE (ADR 0006)
// ============================================================================
// Multi-instance deployments select this store so replicas claim projection
// checkpoints instead of double-processing events. These tests exercise the
// SQL semantics against the real database.
// ============================================================================

async fn stored_sequence(pool: &PgPool, consumer: &str) -> Option<i64> {
    sqlx::query_scalar(
        "SELECT last_global_sequence FROM projection_checkpoints WHERE projection_name = $1",
    )
    .bind(consumer)
    .fetch_optional(pool)
    .await
    .expect("Failed to read checkpoint")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn coordinated_checkpoint_advances_monotonically() {
    let pool = crate::integration::common::database::get_test_pool().await;
    let consumer = format!("coordinated-test-{}", Uuid::new_v4());
    let store = PostgresCoordinatedTxCheckpointStore::new();

    // Fresh consumer: first save inserts.
    let mut tx: Transaction<'static, Postgres> = pool.begin().await.unwrap();
    store.save(&consumer, 5, &mut tx).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(stored_sequence(&pool, &consumer).await, Some(5));

    // Normal path: later sequence advances the checkpoint.
    let mut tx = pool.begin().await.unwrap();
    store.save(&consumer, 9, &mut tx).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(stored_sequence(&pool, &consumer).await, Some(9));

    // Out-of-order save (a later event already advanced the checkpoint):
    // succeeds without regressing the checkpoint.
    let mut tx = pool.begin().await.unwrap();
    store.save(&consumer, 7, &mut tx).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        stored_sequence(&pool, &consumer).await,
        Some(9),
        "checkpoint must never regress"
    );

    sqlx::query("DELETE FROM projection_checkpoints WHERE projection_name = $1")
        .bind(&consumer)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn coordinated_checkpoint_rejects_duplicate_event() {
    let pool = crate::integration::common::database::get_test_pool().await;
    let consumer = format!("coordinated-test-{}", Uuid::new_v4());
    let store = PostgresCoordinatedTxCheckpointStore::new();

    let mut tx = pool.begin().await.unwrap();
    store.save(&consumer, 5, &mut tx).await.unwrap();
    tx.commit().await.unwrap();

    // Another instance committing the same event must fail, so its duplicate
    // projection transaction rolls back.
    let mut tx = pool.begin().await.unwrap();
    assert!(
        store.save(&consumer, 5, &mut tx).await.is_err(),
        "saving an already-committed sequence must fail"
    );
    tx.rollback().await.unwrap();

    // The duplicate detection releases its claim: the next save works.
    let mut tx = pool.begin().await.unwrap();
    store.save(&consumer, 6, &mut tx).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(stored_sequence(&pool, &consumer).await, Some(6));

    sqlx::query("DELETE FROM projection_checkpoints WHERE projection_name = $1")
        .bind(&consumer)
        .execute(&pool)
        .await
        .unwrap();
}
