# CLAUDE.md — Server

Guidance for Claude Code when working on the **Rust backend** in `server/`.

## Overview

Photonic's server is a photo management and processing API written in Rust using the Axum web
framework. It provides RESTful APIs for managing photo albums, media processing, and user
management with JWT-based authentication.

All commands in this file run from `server/`. The Nix devshells (`nix develop`,
`nix develop .#test`, invoked from the repo root) automatically `cd` into `server/` on entry,
so inside a Nix shell you can run `cargo` directly. Outside Nix, `cd server` first.

## Development Commands

### Build and Run

```bash
# Build the project
cargo build

# Run the server (binary name: photonic)
cargo run

# Build optimized release version
cargo build --release
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check code without building
cargo check

# Single crate (fast loop when working on one module)
cargo check -p medium
```

### Database Management

```bash
# Run database migrations (requires DATABASE_URL environment variable)
sqlx migrate run --source crates/composition/migrations

# Revert last migration
sqlx migrate revert --source crates/composition/migrations

# Create new migration
sqlx migrate add <migration_name> --source crates/composition/migrations
```

### Testing

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests of a single module crate
cargo test -p medium

# Integration tests (spin up the real server; need DATABASE_URL)
cargo test -p composition
```

### OpenAPI Spec and Typed Client Generation

The OpenAPI specification is generated from utoipa annotations and written to the repo root
(`../openapi.yaml`). The Swift app symlinks to that same file, so it is the single source of
truth for the API contract.

```bash
# Generate spec and client automatically during build
cargo build

# Manually regenerate just the OpenAPI spec (writes to ../openapi.yaml)
cargo xtask generate-openapi

# View the generated spec (at repo root, shared contract)
cat ../openapi.yaml

# The typed client is generated to:
# target/<profile>/build/photonic-client-<hash>/out/photonic_client.rs
```

**How it works:**

1. **Generate OpenAPI Spec** (manual): Run `cargo xtask generate-openapi` to update
   `../openapi.yaml` from your utoipa annotations. The annotations live in the HTTP adapter
   (`crates/adapters/http`), which xtask calls directly.
2. **Generate Client** (automatic): During `cargo build`, the `build.rs` script in the
   `photonic-client` crate automatically generates a typed Rust client from `../openapi.yaml`
   using progenitor
3. The generated client is available in integration tests via
   `include!(env!("PHOTONIC_CLIENT_PATH"))`

**Why two steps?** Due to Cargo's build system limitations, we can't automatically generate the
spec during build (it would create a circular dependency). The client generation is automatic
because it only reads the YAML file.

**Using the typed client in integration tests:**

```rust
#[rstest]
#[tokio::test]
async fn test_list_media(#[future] app: TestApp, token: String) {
    let media = app.api(&token)
        .client()
        .medium_list()
        .send()
        .await?;

    // Full type safety and IDE autocomplete!
    assert_eq!(media[0].id, expected_id);
}
```

**Modifying the API:**

1. Update the handler function and its `#[utoipa::path(...)]` annotation in
   `crates/adapters/http/src/api/`
2. Update DTOs with `#[derive(ToSchema)]` if adding new types
3. Regenerate the spec: `cargo xtask generate-openapi`
4. Run `cargo build` - the typed client regenerates automatically from the spec
5. Update tests - the compiler will catch breaking changes!

**xtask Commands:**

```bash
# Generate OpenAPI spec manually (writes to ../openapi.yaml by default)
cargo xtask generate-openapi

# Specify custom output location
cargo xtask generate-openapi --output custom-path.yaml
```

## Architecture: Modular Monolith with Hexagonal Adapters

The server is a modular monolith organized around bounded contexts ("modules") plus a shared
kernel, with all technical concerns isolated in adapter crates and wired together by a single
composition root.

### The Dependency Rule

Dependencies only point inward:

```
bin → composition → adapters → modules → kernel → event_sourcing
                         ↓            ↓
                    (implement)   (define ports)
```

- **Modules never depend on each other.** Cross-module collaboration happens either through
  ports wired by the composition root, or through events on the projection bus. (The former
  `medium → user` exception is gone: quota is behind medium's `QuotaPort`, ADR 0005.)
- **Modules never depend on adapters or the composition root.**

### Directory Structure

```
server/
├── Cargo.toml                      # Workspace root
└── crates/
    ├── kernel/                     # Shared vocabulary (NO business logic)
    │                               #   DomainError/ApplicationError, DomainEvent re-export,
    │                               #   PublishEvent port, ID aliases, FileLocation/StorageTier,
    │                               #   file value objects, serde helpers, crypto
    ├── event_sourcing/             # Event store, projection bus, aggregate reconstitution
    │
    ├── modules/                    # Bounded contexts — each owns its domain + application layer
    │   ├── medium/                 #   src/domain/ (aggregates, events, VOs)
    │   │                           #   src/application/ (commands, queries, ports, listeners)
    │   ├── metadata/
    │   ├── task/
    │   ├── user/
    │   └── system/                 #   Application-only (system info query)
    │
    ├── adapters/                   # One crate per external technology
    │   ├── http/                   #   Axum handlers, API DTOs, JWT auth (inbound)
    │   ├── postgres/               #   Repositories, event/snapshot/checkpoint stores, projections
    │   ├── filesystem/             #   File storage (FileStorage port impl)
    │   └── exif/                   #   exiftool metadata extraction (MetadataExtractor port impl)
    │
    ├── composition/                # Composition root (lib) — the ONLY place that knows all
    │   │                           #   concrete types
    │   ├── config/                 #   confique GlobalConfig (server/storage/database)
    │   ├── di/                     #   Container, factories, listener registration, streams
    │   ├── listeners/              #   CROSS-MODULE listeners (see below)
    │   ├── events/                 #   Event bus adapter implementing PublishEvent
    │   ├── migrations/             #   SQL migrations
    │   └── tests/                  #   Integration tests (spin up the real server)
    │
    ├── bin/                        # Thin main(): tracing + config + run_server (package "photonic")
    ├── photonic-client/            # Generated typed client (for integration tests)
    └── xtask/                      # Build automation (OpenAPI spec generation)
```

### Layers

#### Kernel

- **Purpose**: Shared vocabulary every module may use — errors, ID types, storage/file value
  objects, event abstractions, serde helpers.
- **Rule**: Keep it tiny and free of business logic. The moment logic wants to live here, push
  it down into one of the modules. A fat kernel is how modular monoliths rot.

#### Modules (bounded contexts)

- **Purpose**: Core business logic. Each module has its own `domain/` (aggregates, events,
  value objects, domain services) and `application/` (CQRS commands/queries, ports, listeners).
- **Dependencies**: `kernel` and `event_sourcing` only — never another module, never an adapter.
- **Ports**: Repository/external-service traits are defined in the module
  (`application/ports.rs` or `domain/ports.rs`) and implemented by adapters.
- **Domain events stay internal** unless another module legitimately needs to react to them.

#### Adapters

- **Purpose**: Technical implementations of module ports + the HTTP inbound adapter. One crate
  per technology (postgres, filesystem, exif, http).
- **Dependencies**: modules (to implement their ports), kernel, event_sourcing. Never the
  composition root.
- **Postgres adapter** contains the per-module projections (read models) next to the
  repositories, plus the event sourcing stores.
- Adapters take **plain settings structs** (e.g. `FilesystemSettings`, `AuthSettings`), mapped
  from the global config by the composition root — they never depend on `GlobalConfig`.

#### Composition root

- **Purpose**: Assemble the application. Owns configuration, the DI container, listener
  registration, background tasks, db init/migrations, and the server runtime (`run_server`).
- **Cross-module listeners live here** (`src/listeners/`): e.g. "when a Medium was created,
  start metadata extraction", "when metadata was extracted, enrich the medium", task lifecycle
  listeners. These glue modules together via the projection bus without creating module-to-module
  dependencies.
- **Integration tests** live in `tests/` here, using the real wiring via `run_server` plus the
  generated typed client.

### Key Architectural Patterns

1. **Repository Pattern**: Modules define repository interfaces (ports), the postgres adapter
   provides implementations
2. **CQRS**: Separate command (write) and query (read) handlers per module
3. **Event Sourcing**: Domain state reconstituted from an append-only event log; projections
   maintain read models; listeners are checkpointed projection handlers (replay on start)
4. **Domain Events**: Modules publish via the `PublishEvent` port; cross-module reactions are
   wired in the composition root
5. **Value Objects**: Business rules encapsulated in kernel/module value objects
6. **Port/Adapter Pattern**: All external dependencies hidden behind interfaces, one adapter
   crate per technology
7. **Composition Root**: All wiring in `composition`; `bin` is a ~20-line main

### Module Communication Rules

1. **Sync calls across modules** go through ports injected by the composition root (e.g.
   medium's `QuotaPort`, implemented by user's `QuotaManager` via a composition adapter).
2. **Async collaboration** goes through the projection bus (domain events), with listeners
   registered in `composition/src/di/listeners.rs` + implemented in
   `composition/src/listeners/`.
3. **Data ownership**: each module owns its tables/projections; no cross-module joins.

### API Structure

- REST API built with Axum (in the `http` adapter)
- OpenAPI documentation generated with utoipa (available at `/api-docs`)
- JWT authentication using jwt-authorizer
- Versioned API endpoints under `/api/v1/`

### Configuration

The application uses environment-based configuration (confique), defined in
`crates/composition/src/config/`:

- `ServerConfig`: Server host, port, and JWT/OAuth settings
- `StorageConfig`: File storage configuration (paths, quotas, cleanup intervals)
- `DatabaseConfig`: PostgreSQL connection settings

Configuration is loaded from environment variables, with optional `.env` file support (at the
repo root). The composition root maps config slices into plain adapter settings.

### External Dependencies

- **PostgreSQL**: Primary data store with SQLx for async database operations
- **exiftool**: External process for extracting photo metadata
- **OpenTelemetry**: Distributed tracing with OTLP exporter support

## Important Notes

- The application requires a PostgreSQL database and runs migrations on startup
- JWT authentication is required for most API endpoints
- File storage paths are configured through environment variables (see `../.env.example`)
- The exiftool binary must be available in the system PATH
- Tracing can be configured via `RUST_LOG` environment variable
- When adding a new adapter: create a crate under `crates/adapters/`, add it to the workspace
  members + `workspace.dependencies`, implement the port there, and wire it in
  `composition/src/di/`
- When adding a new module: create `crates/modules/<name>/` with `domain/` + `application/`,
  add ports for anything external, and register its streams/projections/listeners in the
  composition root
