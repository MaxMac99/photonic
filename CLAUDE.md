# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Photonic is a photo management and processing API server written in Rust using the Axum web framework. It provides RESTful APIs for managing photo albums, media processing, and user management with JWT-based authentication.

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

# The AsyncAPI spec is located at:
# asyncapi.yaml

# Generated DTOs are output to:
# src/infrastructure/events/generated/
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
1. Edit `asyncapi.yaml` to add/modify event schemas
2. Run `cargo build` - DTOs will be regenerated automatically
3. Implement the `EventDto` trait for the generated struct (manual step)

### OpenAPI Spec and Typed Client Generation

The OpenAPI specification is automatically generated from utoipa annotations during every `cargo build`, and a fully-typed Rust client is generated for use in integration tests.

```bash
# Generate spec and client automatically during build
cargo build

# Manually generate just the OpenAPI spec (without full build)
cargo xtask generate-openapi

# View the generated spec
cat openapi.yaml

# The typed client is generated to:
# target/<profile>/build/photonic-<hash>/out/photonic_client.rs
```

**How it works:**
1. **Generate OpenAPI Spec** (manual): Run `cargo run --example generate_openapi` or `cargo xtask generate-openapi` to update `openapi.yaml` from your utoipa annotations
2. **Generate Client** (automatic): During `cargo build`, the `build.rs` script automatically generates a typed Rust client from `openapi.yaml` using progenitor
3. The generated client is available in integration tests via `include!(env!("PHOTONIC_CLIENT_PATH"))`

**Why two steps?** Due to Cargo's build system limitations, we can't automatically generate the spec during build (it would create a circular dependency). The client generation is automatic because it only reads the YAML file.

**Using the typed client in integration tests:**

```rust
#[rstest]
#[tokio::test]
async fn test_list_media(#[future] app: TestApp, token: String) {
    // Old way (untyped, manual URL construction):
    let response = app.get("/api/v1/medium")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let media: Vec<serde_json::Value> = response.json().await?;

    // New way (typed, auto-generated client):
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
3. Regenerate the spec: `cargo run --example generate_openapi` (or `cargo xtask generate-openapi`)
4. Run `cargo build` - the typed client regenerates automatically from the spec
5. Update tests - the compiler will catch breaking changes!

**xtask Commands:**

```bash
# Generate OpenAPI spec manually
cargo xtask generate-openapi

# Specify custom output location
cargo xtask generate-openapi --output custom-path.yaml
```

**Architecture:**
- **xtask crate**: Separate workspace member that depends on the main `photonic` crate
- **build.rs**: Orchestrates the generation process during compilation
- **progenitor**: Generates idiomatic Rust client code from OpenAPI spec
- **Type safety**: Integration tests get compile-time guarantees for API calls

## Architecture: Clean Hexagonal Architecture

### Architecture Principles

The application follows **Clean Hexagonal Architecture** (Ports and Adapters) with clear separation of concerns:

1. **Dependency Rule**: Dependencies only point inward - outer layers depend on inner layers, never the reverse
2. **Domain Independence**: The domain layer has zero dependencies on infrastructure, frameworks, or external libraries
3. **Port/Adapter Pattern**: All external dependencies are abstracted behind interfaces (ports) with implementations (adapters) in the infrastructure layer
4. **CQRS Pattern**: Application layer uses Command Query Responsibility Segregation for clear separation of read and write operations
5. **Value Objects**: Rich domain models with business rule validation encapsulated in value objects
6. **Dependency Injection**: All dependencies are injected through a central DI container, maintaining loose coupling

### Directory Structure

```
src/
├── domain/                 # Core business logic (innermost layer)
│   ├── album/             # Album bounded context
│   │   ├── entity.rs      # Album entity (aggregate root)
│   │   ├── value_objects.rs # Value objects (AlbumTitle, etc.)
│   │   ├── events.rs      # Domain events
│   │   ├── ports.rs       # Repository & service interfaces
│   │   └── service.rs     # Domain services
│   ├── medium/            # Medium bounded context
│   │   ├── entity.rs      # Medium entity
│   │   ├── value_objects.rs # Value objects
│   │   ├── events.rs      # Domain events
│   │   ├── ports.rs       # Repository & storage interfaces
│   │   ├── service.rs     # Domain services
│   │   └── storage.rs     # Domain storage abstractions
│   └── user/              # User bounded context
│       ├── entity.rs      # User entity
│       ├── value_objects.rs # Value objects (Username, Email, etc.)
│       ├── events.rs      # Domain events
│       ├── ports.rs       # Repository interfaces
│       └── service.rs     # Domain services
│
├── application/           # Use cases / Application services
│   ├── album/            # Album use cases
│   │   ├── commands/     # Write operations (CQRS)
│   │   │   ├── create_album.rs
│   │   │   ├── update_album.rs
│   │   │   └── delete_album.rs
│   │   └── queries/      # Read operations (CQRS)
│   │       ├── get_album.rs
│   │       └── list_albums.rs
│   ├── medium/           # Medium use cases
│   │   ├── commands/
│   │   └── queries/
│   └── user/             # User use cases
│       ├── commands/
│       └── queries/
│
├── infrastructure/        # Technical implementations
│   ├── config/           # Configuration management
│   │   ├── ports.rs      # ConfigProvider trait
│   │   ├── env_adapter.rs # Environment config adapter
│   │   └── test_adapter.rs # Test config adapter
│   ├── di/               # Dependency injection
│   │   └── container.rs  # DI container (composition root)
│   ├── persistence/      # Database implementations
│   │   └── postgres/     # PostgreSQL adapters
│   │       ├── album_repository.rs
│   │       ├── medium_repository.rs
│   │       └── user_repository.rs
│   ├── storage/          # File storage implementations
│   │   ├── filesystem/   # Local filesystem adapter
│   │   ├── s3/          # AWS S3 adapter (future)
│   │   └── adapters.rs  # Storage location mapping
│   ├── events/           # Event bus implementation
│   │   ├── bus.rs        # In-memory event bus
│   │   └── adapters.rs   # Event publisher adapters
│   └── external/         # External service integrations
│       └── exif/         # Exiftool integration
│
├── adapters/             # Inbound adapters (driving adapters)
│   ├── api/              # HTTP API adapters
│   │   ├── album_handler.rs
│   │   ├── medium_handler.rs
│   │   └── user_handler.rs
│   ├── grpc/             # gRPC adapters (future)
│   └── cli/              # CLI adapters (future)
│
├── shared/               # Shared kernel
│   ├── error.rs          # Common error types
│   ├── result.rs         # Common Result type
│   └── types.rs          # Shared value types
│
└── main.rs               # Application entry point
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
- **Purpose**: Technical implementations of domain ports
- **Contains**: Database adapters, File storage, External services, Configuration
- **Dependencies**: Domain and Application layers
- **Example**: `AlbumRepositoryImpl` implements `AlbumRepository` trait using PostgreSQL

#### Adapters Layer (outermost)
- **Purpose**: Entry points to the application
- **Contains**: HTTP handlers, CLI commands, Message queue consumers
- **Dependencies**: Application layer (never domain directly)
- **Example**: `album_handler.rs` translates HTTP requests to application commands

### Key Architectural Patterns

1. **Repository Pattern**: Domain defines repository interfaces (ports), infrastructure provides implementations
2. **CQRS**: Separate command (write) and query (read) models in the application layer
3. **Domain Events**: Enable loose coupling between bounded contexts
4. **Value Objects**: Encapsulate business rules and validation (e.g., `AlbumTitle` ensures max length)
5. **Dependency Injection**: Central DI container wires all dependencies without coupling
6. **Port/Adapter Pattern**: All external dependencies hidden behind interfaces

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

Configuration is loaded from environment variables, with optional `.env` file support.

### External Dependencies

- **PostgreSQL**: Primary data store with SQLx for async database operations
- **exiftool**: External process for extracting photo metadata
- **OpenTelemetry**: Distributed tracing with OTLP exporter support

## Important Notes

- The application requires a PostgreSQL database and runs migrations on startup
- JWT authentication is required for most API endpoints
- File storage paths are configured through environment variables
- The exiftool binary must be available in the system PATH
- Tracing can be configured via RUST_LOG environment variable