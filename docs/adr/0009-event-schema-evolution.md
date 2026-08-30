# ADR 0009: Event schema evolution is additive-only

## Context

Events are stored forever and replayed. `EventTypeRegistry` maps type names
to Rust types; payloads deserialize straight into current structs. Nothing
guards against drifting old stored payloads away from today's types.

## Decision

- Events are **immutable facts**. Payloads are never mutated in place and
  serde semantics of a published event type never change silently.
- A breaking event change publishes a **new versioned event type**
  (`MediumCreatedV2`) with its own registry/stream/projection wiring; the
  old type and deserializer stay for replay. Readers branch on version
  where needed.
- A **replay regression test** deserializes every event in the event store
  against the current registry, so drift fails CI instead of production
  replay.

## Consequences

- No upcasting infrastructure until the first painful, common event change
  makes an upcasting chain worthwhile (a registry upgrade, not a redesign).
- Reader code may branch on V1/V2 events occasionally — acceptable at
  current scale (~20 event types).
- The replay test is the tripwire that keeps stored history loadable.
