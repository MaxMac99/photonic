use rstest::rstest;
use serial_test::serial;
use sqlx::Row;

use crate::integration::{
    common::fixtures::{image, user},
    test_app::TestApp,
};

// ============================================================================
// EVENT REPLAY REGRESSION TEST (ADR 0009)
// ============================================================================
// Guards against event schema drift: every event stored in the event store
// must deserialize with the CURRENT type registry. If a published event type
// changes serde semantics or gets renamed/removed, this test fails — the
// stored history would otherwise become unloadable.
// ============================================================================

async fn read_events(pool: &sqlx::PgPool) -> Vec<(String, serde_json::Value, i64)> {
    sqlx::query("SELECT event_type, payload, global_sequence FROM events ORDER BY global_sequence")
        .fetch_all(pool)
        .await
        .expect("Failed to read events")
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("event_type"),
                row.get::<serde_json::Value, _>("payload"),
                row.get::<i64, _>("global_sequence"),
            )
        })
        .collect()
}

/// Replays every stored event through the current event type registry.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn all_stored_events_deserialize_with_current_registry() {
    let pool = crate::integration::common::database::get_test_pool().await;

    let mut events = read_events(&pool).await;

    // Seed a realistic event mix (user/quota/medium/metadata events) through
    // the real API if the store is empty, so the test is never vacuous.
    if events.is_empty() {
        let app = TestApp::new().await;
        let image = image("IMG_4598.HEIC");
        let user = user(10_000_000_000);
        let _ = app.create_medium(&user, image.into()).await;
        drop(app);
        events = read_events(&pool).await;
    }

    assert!(
        !events.is_empty(),
        "Event store is empty even after seeding — replay test is vacuous."
    );

    let (_bus, registry) = composition::di::event_system::build_projection_bus(&pool)
        .expect("Failed to build projection bus / registry");

    let registered: Vec<String> = registry
        .write()
        .unwrap()
        .event_types()
        .into_iter()
        .map(String::from)
        .collect();
    let registered: Vec<&str> = registered.iter().map(String::as_str).collect();

    let mut unknown_types = Vec::new();
    let mut failures = Vec::new();

    for (event_type, payload, sequence) in &events {
        if !registered.contains(&event_type.as_str()) {
            unknown_types.push(event_type.clone());
            continue;
        }
        if let Err(e) = registry.read().unwrap().deserialize(event_type, payload) {
            failures.push(format!(
                "event {sequence} of type '{event_type}' failed to deserialize: {e}"
            ));
        }
    }

    assert!(
        unknown_types.is_empty(),
        "Stored event types not registered in the current registry: {unknown_types:?}. \
         Old events would fail to replay."
    );
    assert!(
        failures.is_empty(),
        "Stored events failed to deserialize against current types (schema drift):\n{}",
        failures.join("\n")
    );
}
