# ADR 0008: Authorization is domain logic, not database enforcement

## Context

Every port method takes `UserId` and repositories filter by ownership
(`WHERE owner_id = $`). Album *and* medium will need sharing ("who can view
this medium"), so permissions are a general business concern across
contexts — not an album special case. Postgres RLS was considered.

## Decision

- Permissions (ownership, sharing, access grants) are **domain/application
  logic**: access policies in modules, share grants as domain aggregates,
  checked in application handlers.
- No Postgres RLS: it would duplicate business rules as SQL policies — a
  second source of truth about who may see what. The database stays a dumb
  store; the domain decides visibility.
- Ownership scoping in repositories stays conventional (explicit `user_id`
  parameters); no scope-typing refactor. When sharing is designed, an
  access policy check happens in application handlers before scoped
  queries run.

## Consequences

- One source of truth for permissions (the domain).
- Every new query must remember the ownership clause — enforced by
  convention and review until sharing arrives with explicit policy
  objects.
- RLS can still be added later as pure defense-in-depth if ever needed;
  nothing in the current design blocks it.
