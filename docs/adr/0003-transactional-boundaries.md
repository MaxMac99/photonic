# ADR 0003: Transactional boundaries in a modular monolith

## Context

Cross-aggregate flows exist (upload = quota reserve + file write + medium
event). There is exactly one Postgres, so we own a single ACID boundary —
the main advantage a modular monolith has over services. The filesystem,
however, can never join a database transaction, so any flow touching files
must tolerate partial failure regardless of the transaction model. The
existing quota manager already does reserve/compensate; the known leak is
that a crash between reserve and commit strands the reservation.

## Decision

- The database is the unit of work; we do not build a Unit-of-Work
  abstraction in the application layer.
- For the rare command that must atomically touch two modules' state, one
  Postgres transaction may span both modules' writes. The transaction is
  scoped in the **postgres adapter / composition root** (e.g. a unit-of-work
  handle yielding transaction-scoped repository implementations); module
  application code stays transaction-agnostic.
- Compensation (reserve → do → commit/release) applies **only at the
  filesystem boundary**, because files cannot join a database transaction.
  The temp-then-promote flow plus the cleanup sweep are that compensation.
- Quota reservations become short-lived soft locks: they get an **expiry**
  (sweeper releases stale reservations) instead of being ledger entries.
- Saga/process-manager machinery is rejected: with one database and
  in-process modules, eventual consistency is not required for DB-only
  cross-module commands. If a module is ever extracted to a service, that
  is the moment compensation choreography becomes relevant.

## Consequences

- Few cross-module commands declare a transaction scope explicitly; most
  commands stay single-aggregate.
- Filesystem-side partial failures remain inherent and are handled by
  sweeps, not transactions.
- Quota correctness no longer depends on the process staying alive:
  stale reservations expire instead of leaking.
