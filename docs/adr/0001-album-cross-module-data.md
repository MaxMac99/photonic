# ADR 0001: Album module gets medium data via a hybrid strategy

Date: 2026-08-30
Status: Accepted

## Context

The album module (not yet implemented) references media (cover, contents,
ordering). Album is a separate bounded context, so it cannot depend on the
medium module. The options were events-only, sync-port-only, or a mix.

## Decision

Hybrid:

- **Album listing (hot path)**: album owns membership, ordering and its own
  denormalized projection (`album_media`) fed by medium integration events
  (`MediumCreated`, `MediumUpdated`, `MediumDeleted`). A few fields only
  (title, taken-at, preview readiness).
- **Single album view**: album calls a narrow sync port
  (`MediaSummaryPort`) that composition wires to medium's query side for
  current details. Dangling ids are filtered at read time.
- **Media deletion**: medium gains a `MediumDeletedEvent`; album tolerates
  dangling ids at read time (filter on read) and may clean up via listener.

## Consequences

- Album is extractable; medium never learns about albums.
- Eventual consistency on list views (acceptable for photo browsing).
- The sync port makes single-album views always fresh without N-way
  denormalization.
- This pattern is the template for future cross-module read needs.
