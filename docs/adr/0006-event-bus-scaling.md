# ADR 0006: Event bus stays a port; Postgres adapter with a
# coordination-safe variant

## Context

The `ProjectionEventBus` is in-process, backed by Postgres (checkpointed,
LISTEN/NOTIFY inside one process). Two replicas would double-process events
unless checkpoint claiming is lock-coordinated. An external broker (NATS,
Kafka) would solve multi-node but adds an operational dependency.

## Decision

- The bus stays behind its port (event_sourcing abstractions); the
  Postgres-backed in-process bus is the **default adapter**, tuned for
  single-instance performance (current fast path: transactional checkpoint
  store, no lock overhead).
- A **coordination-safe** checkpoint store variant lives in the same
  postgres adapter (checkpoint claims via `FOR UPDATE SKIP LOCKED` /
  advisory locks). It is selected by configuration
  (`server.multi_instance = true`), not by a second implementation crate.
- No external broker: multi-instance is achieved with the same Postgres,
  trading some latency for coordination safety.

## Consequences

- Default deployment keeps zero extra latency and no external service.
- Running two instances requires only a config flag; they coordinate on
  checkpoints instead of corrupting them.
- Extraction/replication stays possible without touching modules; the
  `EventBus` port remains the seam.
