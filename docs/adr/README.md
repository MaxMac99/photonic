# Architecture Decision Records

Decisions for the Photonic server (modular monolith, hexagonal adapters).
Status `Accepted` decisions are binding for new code; revisiting one means
writing a new ADR that supersedes the old.

| ADR | Decision |
| --- | --- |
| [0001](0001-album-cross-module-data.md) | Album module gets medium data via hybrid (events projection + narrow sync port) |
| [0002](0002-cqrs-read-side.md) | Full CQRS split: dedicated query ports over projection tables |
| [0003](0003-transactional-boundaries.md) | Shared-DB transactions for cross-module commands; compensation only at the filesystem boundary |
| [0004](0004-event-contract.md) | Rust event types + docs/events.md are the contract; no AsyncAPI until a consumer exists |
| [0005](0005-quota-port.md) | Quota behind a QuotaPort owned by the medium module |
| [0006](0006-event-bus-scaling.md) | Bus stays a port; Postgres adapter with coordination-safe variant for multi-instance |
| [0007](0007-storage-backend-routing.md) | Tier-keyed composite storage (NVMe temp/cache, S3 permanent later); cache = read-through cache |
| [0008](0008-authorization.md) | Authorization is domain logic; no RLS; conventional owner scoping until sharing |
| [0009](0009-event-schema-evolution.md) | Events are immutable; breaking changes add versioned types; replay regression test guards drift |
