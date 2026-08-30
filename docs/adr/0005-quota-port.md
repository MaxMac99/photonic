# ADR 0005: Quota behind a QuotaPort in the medium module

## Context

`medium` depends on the `user` module for `QuotaManager` — the single
module-to-module crate edge. `QuotaManager::with_quota` takes a generic
closure and therefore cannot be expressed as an object-safe port. ADR 0003
resigns quota reservations as short-lived soft locks with expiry.

## Decision

- `medium` defines a **`QuotaPort`**: `reserve(user_id, bytes) ->
  Reservation`, `commit(Reservation)`, `release(reservation)` returning an
  opaque `Reservation` handle.
- The `user` module's `QuotaManager` implements the port; the composition
  root injects `Arc<dyn QuotaPort>` into medium's handlers.
- `medium` loses its dependency on `user`; the module graph becomes a star
  around the composition root again.
- Reservation/expiry semantics live inside the port implementation, not in
  callers.

## Consequences

- Explicit reserve/commit/release call sites replace the closure dance.
- Medium is standalone; album can reuse the same port for uploads into
  albums.
- Implemented together with the reservation-expiry sweep (ADR 0003) so the
  quota model changes once.
