# ADR 0004: No AsyncAPI contract; Rust types + docs are the event contract

## Context

A shared `asyncapi.yaml` at the repo root has been referenced in CLAUDE.md,
AGENTS.md and CI path filters — but the file never existed in git history
and nothing consumes it. The de-facto event contract is the Rust event
types in `modules/*/src/domain/events/` plus the hand-maintained catalog in
`docs/events.md`. The Apple app talks REST only; the bus is in-process over
Postgres; there is no external event consumer.

A contract file that nothing validates drifts into fiction (this one never
existed at all). Hand-written schema files next to hand-written docs are
two fictions instead of one.

## Decision

- Drop the asyncapi pretense: remove stale references from CLAUDE.md,
  AGENTS.md and CI path filters.
- The event contract **is** the Rust types in
  `modules/*/src/domain/events/` (compiler-enforced shape) plus
  `docs/events.md` (flows, intent).
- When a real second event consumer appears (SSE/WebSocket push to clients,
  an extracted service, external integrations), generate the contract
  artifact **from the Rust types** (utoipa-style) instead of hand-writing
  it. Never hand-sync a contract file.

## Consequences

- No phantom build step; docs match reality.
- Cross-process event publishing requires a deliberate contract step later,
  generated from code, versioned like the OpenAPI flow.
