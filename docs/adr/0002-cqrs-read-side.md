# ADR 0002: Full CQRS split on the read side

## Context

Today queries and writes share one port per module (`MediumRepository`
carries `save`/`delete`, command-support reads like `get_user_usage`, and
query-shaped reads like `find_all` returning list DTOs). Projections already
maintain read-optimized tables (`media`, `medium_items`, `locations`), and
the query SQL already targets them — but the port still presents everything
as one "repository".

## Decision

Full CQRS split:

- Each module gets dedicated **query ports** in
  `modules/<m>/application/queries` with per-use-case query objects
  returning shaped DTOs, backed by SQL against the projection tables.
- The aggregate repository ports keep **write-side only** operations
  (save, delete) plus command-support reads they genuinely need.
- The HTTP adapter depends on query ports for reads and command handlers
  for writes; never on the repository for shaped reads.
- New read models (e.g. the album projection) follow the same shape:
  projection tables + query ports, one port per use-case family.

## Consequences

- Ports and signatures churn once (mechanical).
- Query side stops pretending to be domain persistence; handlers depend on
  exactly what they use.
- Sets the pattern the album module's second read model will follow.
