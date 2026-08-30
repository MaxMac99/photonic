# ADR 0007: Tier-keyed composite storage adapter

## Context

`FileLocation`/`StorageTier` (Permanent, Temporary, Cache) live in kernel;
the `FileStorage` port is implemented by the filesystem adapter, which maps
tier → local directory. The intended target topology is per-tier backend
affinity: e.g. Cache/Temporary on local NVMe, Permanent on S3/object
storage.

## Decision

- Keep the **single `FileStorage` port**. Add a **composite storage
  adapter** that dispatches each call on `location.storage_tier` to a
  per-backend adapter (Permanent → object store, Temporary/Cache → local
  filesystem). Built in the composition root from configuration.
- **Cross-backend moves** (temp → permanent promotion, i.e. every upload)
  are streaming copies between backends, deleting the source after
  success. Crash-safe by nature: worst case is an orphaned temp file,
  which the existing sweep reclaims.
- The **Cache tier doubles as the read-through cache** for cold objects:
  `get_local_path` on a permanent-tier object materializes it into the
  Cache tier and returns that path.
- Modules must never learn backend specifics — a command may never ask
  "am I on S3?"; the tier is the only routing key that exists in domain
  types.

## Consequences

- The filesystem adapter stays as-is and may keep owning Temporary and
  Cache even when Permanent moves to S3.
- `get_local_path`'s contract is "materialized locally (possibly via the
  cache tier)" — object-store adapters implement it via download-on-demand.
- The composite adapter is swappable without touching kernel or modules.
