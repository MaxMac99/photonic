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

# Run in development mode
cargo run

# Build optimized release version
cargo build --release

# Run release build
cargo run --release
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check code without building
cargo check
```

### Database Management

```bash
# Run database migrations (requires DATABASE_URL environment variable)
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Create new migration
sqlx migrate add <migration_name>
```

### Testing

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test <test_name>
```

### Event DTOs and AsyncAPI

Event DTOs are automatically generated from the AsyncAPI specification during the build process.

```bash
# Event DTOs are generated automatically during cargo build
cargo build

# The AsyncAPI spec is located at (repo root, shared contract):
# ../asyncapi.yaml   (from server/)

# Generated DTOs are output to:
# crates/infrastructure/src/infrastructure/events/generated/
```

**How it works:**

1. The `build.rs` script runs before compilation
2. It reads `asyncapi.yaml` and uses the AsyncAPI CLI to generate Rust DTOs
3. Generated DTOs have serde support for serialization/deserialization
4. The build script only regenerates when `asyncapi.yaml` changes

**Prerequisites:**

- AsyncAPI CLI is automatically installed via Nix flake's shellHook
- If not using Nix: `npm install -g @asyncapi/cli`

**Modifying events:**

1. Edit `asyncapi.yaml` (at the repo root) to add/modify event schemas
2. Run `cargo build` - DTOs will be regenerated automatically
3. Implement the `EventDto` trait for the generated struct (manual step)

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
   `../openapi.yaml` from your utoipa annotations
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

1. Update the handler function and its `#[utoipa::path(...)]` annotation
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

## Architecture: Clean Hexagonal Architecture

### Architecture Principles

The application follows **Clean Hexagonal Architecture** (Ports and Adapters) with clear
separation of concerns:

1. **Dependency Rule**: Dependencies only point inward - outer layers depend on inner layers,
   never the reverse
2. **Domain Independence**: The domain layer has zero dependencies on infrastructure, frameworks,
   or external libraries
3. **Port/Adapter Pattern**: All external dependencies are abstracted behind interfaces (ports)
   with implementations (adapters) in the infrastructure layer
4. **CQRS Pattern**: Application layer uses Command Query Responsibility Segregation for clear
   separation of read and write operations
5. **Value Objects**: Rich domain models with business rule validation encapsulated in value
   objects
6. **Dependency Injection**: All dependencies are injected through a central DI container,
   maintaining loose coupling

### Directory Structure

```
server/
├── Cargo.toml                      # Workspace root
└── crates/
    ├── domain/                     # Core business logic (innermost layer)
    │   ├── album/                  # Album bounded context
    │   ├── medium/                 # Medium bounded context
    │   └── user/                   # User bounded context
    │
    ├── application/                # Use cases / Application services (CQRS)
    │   ├── album/
    │   │   ├── commands/           # Write operations
    │   │   └── queries/            # Read operations
    │   ├── medium/
    │   └── user/
    │
    ├── event_sourcing/             # Event store, projections, aggregate reconstitution
    │
    ├── infrastructure/             # Technical implementations
    │   ├── config/                 # Configuration management
    │   ├── di/                     # Dependency injection container
    │   ├── persistence/postgres/   # PostgreSQL adapters
    │   ├── storage/                # File storage implementations
    │   ├── events/                 # Event bus implementation
    │   ├── api/                    # Axum HTTP handlers (inbound adapter)
    │   └── external/exif/          # Exiftool integration
    │
    ├── photonic-client/            # Generated typed client (for integration tests)
    │
    └── xtask/                      # Build automation (OpenAPI spec generation)
```

### Layer Responsibilities

#### Domain Layer (innermost)

- **Purpose**: Core business logic and rules
- **Contains**: Entities, Value Objects, Domain Services, Domain Events, Port interfaces
- **Dependencies**: NONE (pure Rust, no external dependencies)
- **Example**: `AlbumService` validates business rules for album operations

#### Application Layer

- **Purpose**: Orchestrates domain objects to fulfill use cases
- **Contains**: Application Services, Command/Query handlers, DTOs
- **Dependencies**: Domain layer only
- **Example**: `CreateAlbumCommand` coordinates album creation using domain services

#### Infrastructure Layer

- **Purpose**: Technical implementations of domain ports + HTTP inbound adapter
- **Contains**: Database adapters, File storage, External services, Configuration, HTTP handlers
- **Dependencies**: Domain and Application layers
- **Example**: `AlbumRepositoryImpl` implements `AlbumRepository` trait using PostgreSQL

### Key Architectural Patterns

1. **Repository Pattern**: Domain defines repository interfaces (ports), infrastructure provides
   implementations
2. **CQRS**: Separate command (write) and query (read) models in the application layer
3. **Event Sourcing**: Domain state reconstituted from an append-only event log; projections
   maintain read models
4. **Domain Events**: Enable loose coupling between bounded contexts
5. **Value Objects**: Encapsulate business rules and validation (e.g., `AlbumTitle` ensures max
   length)
6. **Dependency Injection**: Central DI container wires all dependencies without coupling
7. **Port/Adapter Pattern**: All external dependencies hidden behind interfaces

### API Structure

- REST API built with Axum framework
- OpenAPI documentation generated with utoipa (available at `/api-docs`)
- JWT authentication using jwt-authorizer
- Versioned API endpoints under `/api/v1/`

### Configuration

The application uses environment-based configuration with the following structure:

- `ServerConfig`: Server host, port, and JWT settings
- `StorageConfig`: File storage configuration
- `DatabaseConfig`: PostgreSQL connection settings

Configuration is loaded from environment variables, with optional `.env` file support (at the
repo root).

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
