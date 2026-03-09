# Photonic System Architecture

This document describes the system architecture of Photonic, including high-level diagrams, component structures, and data flows.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Clean Hexagonal Architecture](#clean-hexagonal-architecture)
- [System Context Diagram (C4)](#system-context-diagram-c4)
- [Container Diagram (C4)](#container-diagram-c4)
- [Component Diagrams](#component-diagrams)
- [Processing Pipeline](#processing-pipeline)
- [Storage Architecture](#storage-architecture)
- [Database Schema](#database-schema)
- [State Machines](#state-machines)
- [Deployment Architecture](#deployment-architecture)

---

## Architecture Overview

Photonic follows **Clean Hexagonal Architecture** (Ports and Adapters) with clear separation of concerns across four main layers.

### Architecture Principles

1. **Dependency Inversion** - Dependencies point inward, outer layers depend on inner layers
2. **Independence of Frameworks** - Domain logic independent of Axum, SQLx, etc.
3. **Testability** - Business logic testable without infrastructure
4. **Independence of UI** - Can swap API with GraphQL, gRPC, CLI
5. **Independence of Database** - Can swap PostgreSQL with other databases
6. **Independence of External Services** - External services abstracted behind ports

### The Hexagon

```
                    ┌─────────────────────┐
                    │                     │
              ┌────►│   HTTP Handlers     │────┐
              │     │   (Driving Adapter) │    │
              │     └─────────────────────┘    │
              │                                 │
    ┌─────────┴──────────┐         ┌───────────▼────────┐
    │                    │         │                    │
    │  Application Layer │◄───────►│   Domain Layer     │
    │    (Use Cases)     │         │  (Business Logic)  │
    │                    │         │                    │
    └─────────┬──────────┘         └───────────▲────────┘
              │                                 │
              │     ┌─────────────────────┐    │
              └────►│  Repository Impls   │────┘
                    │  (Driven Adapter)   │
                    └─────────────────────┘
```

---

## Clean Hexagonal Architecture

### Layer Structure

```
┌─────────────────────────────────────────────────────────┐
│                   API Layer (Outermost)                 │
│  ┌────────────┐  ┌──────────┐  ┌──────────────┐       │
│  │  HTTP      │  │   gRPC   │  │     CLI      │       │
│  │  Handlers  │  │  (Future)│  │   (Future)   │       │
│  └──────┬─────┘  └─────┬────┘  └──────┬───────┘       │
└─────────┼──────────────┼───────────────┼───────────────┘
          │              │               │
┌─────────▼──────────────▼───────────────▼───────────────┐
│             Application Layer (Use Cases)               │
│  ┌──────────────┐              ┌──────────────┐       │
│  │   Commands   │              │   Queries    │       │
│  │              │              │              │       │
│  │ - CreateMed  │              │ - GetMedium  │       │
│  │ - DeleteMed  │              │ - ListMedia  │       │
│  │ - AddTags    │              │ - SearchMed  │       │
│  └──────┬───────┘              └──────┬───────┘       │
│         │                             │               │
│  ┌──────▼─────────────────────────────▼───────┐       │
│  │        Event Listeners (Async)             │       │
│  │  - ExifExtractor                           │       │
│  │  - FileM over                              │       │
│  │  - VariantGenerators                       │       │
│  └────────────────────────────────────────────┘       │
└─────────┼──────────────────────────────────────────────┘
          │
┌─────────▼──────────────────────────────────────────────┐
│              Domain Layer (Pure Business Logic)         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   User      │  │   Medium     │  │    Album     │ │
│  │  Domain     │  │   Domain     │  │   Domain     │ │
│  │             │  │              │  │              │ │
│  │ - Entity    │  │ - Entity     │  │ - Entity     │ │
│  │ - ValueObjs │  │ - ValueObjs  │  │ - ValueObjs  │ │
│  │ - Events    │  │ - Events     │  │ - Events     │ │
│  │ - Services  │  │ - Services   │  │ - Services   │ │
│  │ - Ports     │  │ - Ports      │  │ - Ports      │ │
│  └─────────────┘  └──────────────┘  └──────────────┘ │
└─────────┼──────────────────────────────────────────────┘
          │
┌─────────▼──────────────────────────────────────────────┐
│          Infrastructure Layer (Technical Details)       │
│  ┌───────────────────────────────────────────────┐    │
│  │              Event Bus (In-Memory)            │    │
│  │         (NATS, Kafka in future)               │    │
│  └────────┬──────────────────────┬─────────────┘     │
│           │                      │                    │
│  ┌────────▼──────────┐  ┌────────▼────────────┐      │
│  │   Repositories    │  │    File Storage     │      │
│  │  - PostgreSQL     │  │  - Filesystem       │      │
│  │    Adapters       │  │  - S3 (Future)      │      │
│  └────────┬──────────┘  └────────┬────────────┘      │
│           │                      │                    │
│  ┌────────▼──────────────────────▼────────────┐      │
│  │           External Services                │      │
│  │  - exiftool (Metadata Extraction)          │      │
│  │  - ML/AI Services (Future)                 │      │
│  └────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────┘
```

### Dependencies Flow

```
API Layer
    ↓ depends on
Application Layer
    ↓ depends on
Domain Layer (depends on nothing)
    ↑ implemented by
Infrastructure Layer
```

**Key Rules:**
- Domain layer has ZERO dependencies
- Infrastructure implements domain ports (interfaces)
- Application orchestrates domain entities
- API translates HTTP to application commands

---

## System Context Diagram (C4)

### Level 1: System Context

```mermaid
graph TB
    User[User<br/>Web/Mobile Client]
    IDP[OAuth2 IDP<br/>Keycloak/Auth0]
    Photonic[Photonic System<br/>Photo Management API]
    DB[(PostgreSQL<br/>Database)]
    Storage[File Storage<br/>Filesystem/S3]
    ExifTool[exiftool<br/>External Process]

    User -->|Uploads photos,<br/>Views media| Photonic
    IDP -->|JWT Tokens<br/>with quota| Photonic
    Photonic -->|Stores metadata| DB
    Photonic -->|Stores files| Storage
    Photonic -->|Extracts EXIF| ExifTool

    style Photonic fill:#4A90E2,stroke:#2E5C8A,color:#fff
    style User fill:#7ED321,stroke:#5FA019
    style IDP fill:#F5A623,stroke:#C47E00
```

**External Systems:**
- **Users** - Web/mobile apps (future) consuming REST API
- **OAuth2 IDP** - Provides authentication and quota information
- **PostgreSQL** - Stores all structured data
- **File Storage** - Stores actual media files
- **exiftool** - External process for metadata extraction

---

## Container Diagram (C4)

### Level 2: Containers

```mermaid
graph TB
    subgraph Client["Client Applications"]
        WebApp[Web Application<br/>React/Vue<br/>Future]
        MobileApp[Mobile App<br/>iOS/Android<br/>Future]
        CLI[CLI Tool<br/>Future]
    end

    subgraph PhotonicSystem["Photonic System"]
        API[API Server<br/>Axum + Rust<br/>Port 8080]
        EventBus[Event Bus<br/>In-Memory<br/>Async Channels]

        subgraph Domain["Domain Layer"]
            UserDomain[User Context]
            MediumDomain[Medium Context]
            AlbumDomain[Album Context]
        end
    end

    IDP[OAuth2 Provider<br/>Keycloak/Auth0]
    DB[(PostgreSQL<br/>Database)]
    FileStore[(File Storage<br/>Temp/Permanent/Cache)]
    ExifTool[exiftool<br/>External Process]

    WebApp -->|HTTPS/JSON| API
    MobileApp -->|HTTPS/JSON| API
    CLI -->|HTTPS/JSON| API

    API -->|Validates JWT| IDP
    API -->|Commands/Queries| Domain
    Domain -->|Publishes Events| EventBus
    EventBus -->|Triggers| Domain
    API -->|SQL Queries| DB
    API -->|File Operations| FileStore
    API -->|Spawns Process| ExifTool

    style API fill:#4A90E2,stroke:#2E5C8A,color:#fff
    style EventBus fill:#F5A623,stroke:#C47E00
    style Domain fill:#7ED321,stroke:#5FA019
```

**Containers:**
- **API Server** - Axum-based REST API (current)
- **Event Bus** - In-memory event distribution (current)
- **Domain Layer** - Pure business logic
- **PostgreSQL** - Relational database
- **File Storage** - Tiered file storage system
- **exiftool** - EXIF metadata extractor

---

## Component Diagrams

### API Layer Components

```mermaid
graph LR
    subgraph API["API Layer"]
        Router[Axum Router]
        Middleware[JWT Middleware]
        UserHandler[User Handlers]
        MediumHandler[Medium Handlers]
        AlbumHandler[Album Handlers]
        SystemHandler[System Handlers]
    end

    subgraph App["Application Layer"]
        Commands[Command Handlers]
        Queries[Query Handlers]
    end

    Client[HTTP Client] --> Router
    Router --> Middleware
    Middleware --> UserHandler
    Middleware --> MediumHandler
    Middleware --> AlbumHandler
    Middleware --> SystemHandler

    UserHandler --> Commands
    UserHandler --> Queries
    MediumHandler --> Commands
    MediumHandler --> Queries
    AlbumHandler --> Commands
    AlbumHandler --> Queries

    style Router fill:#4A90E2,color:#fff
    style Middleware fill:#F5A623
```

**Components:**
- **Router** - Route matching and dispatching
- **Middleware** - JWT validation, logging, tracing
- **Handlers** - HTTP request/response translation
- **Commands/Queries** - CQRS pattern implementation

### Application Layer Components

```mermaid
graph TB
    subgraph Commands["Commands (Write Operations)"]
        CreateMedium[CreateMediumStreamCommand]
        DeleteMedium[DeleteMediumCommand]
        AddTags[AddTagsCommand]
        EnsureUser[EnsureUserExistsCommand]
    end

    subgraph Queries["Queries (Read Operations)"]
        GetMedium[GetMediumQuery]
        ListMedia[ListMediaQuery]
        SearchMedia[SearchMediaQuery]
        GetQuota[GetQuotaQuery]
    end

    subgraph Listeners["Event Listeners"]
        ExifListener[ExifExtractorListener]
        MoveListener[FileMovierListener]
        ThumbnailGen[ThumbnailGeneratorListener]
        PreviewGen[PreviewGeneratorListener]
    end

    EventBus[Event Bus]

    Commands --> Domain[Domain Layer]
    Queries --> Domain
    Domain --> EventBus
    EventBus --> Listeners
    Listeners --> Domain

    style Commands fill:#E74C3C,color:#fff
    style Queries fill:#3498DB,color:#fff
    style Listeners fill:#2ECC71,color:#fff
```

---

## Processing Pipeline

### Upload and Processing Flow

```mermaid
sequenceDiagram
    participant U as User
    participant API as API Handler
    participant QS as QuotaService
    participant MR as MediumRepo
    participant FS as FileStorage
    participant EB as EventBus
    participant EXIF as ExifListener
    participant Move as MoveListener
    participant VG as VariantGen

    U->>API: POST /media (upload)
    API->>QS: reserve_quota(size)
    QS-->>API: Reservation OK
    QS->>EB: QuotaReserved

    par Store File & Create Record
        API->>FS: store_file_stream(temp)
    and
        API->>MR: create(Medium)
    end

    API->>EB: publish(MediumUploaded)
    API-->>U: 201 Created {medium_id}

    Note over EB,EXIF: Async Processing Starts

    EB->>EXIF: MediumUploaded event
    EXIF->>EXIF: extract_metadata()
    EXIF->>MR: update_metadata()
    EXIF->>EB: MediumMetadataExtracted

    EB->>Move: MediumMetadataExtracted
    Move->>Move: calculate_path()
    Move->>FS: move_file(temp→permanent)
    Move->>MR: create_medium_item(Original)
    Move->>QS: commit_reservation()
    QS->>EB: QuotaCommitted
    Move->>EB: MediumMovedToFinal

    par Generate Variants
        EB->>VG: Generate Thumbnail
        VG->>FS: store(thumbnail)
        VG->>EB: ThumbnailCreated
    and
        EB->>VG: Generate Preview
        VG->>FS: store(preview)
        VG->>EB: PreviewCreated
    and
        EB->>VG: Generate LowRes (if RAW)
        VG->>FS: store(lowres)
        VG->>EB: LowResCreated
    end

    EB->>EB: Check completion
    EB->>EB: publish(MediumReady)
```

### State Transitions

```mermaid
stateDiagram-v2
    [*] --> Uploading: User uploads file
    Uploading --> Processing: MediumUploaded
    Processing --> Ready: All processing complete
    Processing --> Failed: Processing error
    Ready --> [*]: MediumDeleted
    Failed --> Processing: Retry
    Failed --> [*]: MediumDeleted
```

---

## Storage Architecture

### Storage Tier Hierarchy

```mermaid
graph TB
    subgraph StorageSystem["File Storage System"]
        Interface[Storage Port<br/>FileStorage trait]

        subgraph Tiers["Storage Tiers"]
            Temp[Temporary<br/>SSD, Fast<br/>1 day TTL]
            Perm[Permanent<br/>HDD/S3<br/>Originals]
            Cache[Cache<br/>SSD<br/>Variants]
            Archive[Archive<br/>Cold Storage<br/>Future]
        end

        subgraph Providers["Storage Providers"]
            FS[Filesystem<br/>Provider]
            S3[S3 Provider<br/>Future]
        end
    end

    Upload[Uploaded File] --> Temp
    Temp -->|After processing| Perm
    Perm -->|Generate variants| Cache
    Perm -->|Rarely accessed| Archive

    Interface --> Tiers
    Tiers --> Providers

    style Temp fill:#E74C3C,color:#fff
    style Perm fill:#3498DB,color:#fff
    style Cache fill:#F39C12,color:#fff
    style Archive fill:#95A5A6,color:#fff
```

### Directory Structure

```
/storage/
├── temporary/              # Fast storage (SSD)
│   ├── {uuid}.jpg          # Uploaded files
│   ├── {uuid}.cr2
│   └── {uuid}.mp4
│
├── permanent/              # Main storage (HDD/S3)
│   ├── 2024/
│   │   ├── 12/
│   │   │   ├── Canon/
│   │   │   │   ├── IMG_1234.jpg
│   │   │   │   ├── IMG_1235.cr2
│   │   │   │   └── IMG_1236.jpg
│   │   │   ├── Sony/
│   │   │   │   └── DSC_0001.arw
│   │   │   └── Unknown/
│   │   │       └── photo.jpg
│   │   ├── 11/
│   │   └── 10/
│   └── 2023/
│       └── ...
│
└── cache/                  # Generated variants (SSD)
    ├── {medium_id}_thumb.jpg
    ├── {medium_id}_preview.jpg
    ├── {medium_id}_lowres.jpg
    └── ...
```

**Path Pattern Examples:**
```
Pattern: {year}/{month}/{camera_make}/{filename}
Result:  2024/12/Canon/IMG_1234.jpg

Pattern: {user_id}/{year}/{camera_model}/{filename}
Result:  550e8400-e29b-41d4-a716-446655440000/2024/EOS_R5/IMG_1234.jpg
```

---

## Database Schema

### Entity Relationship Diagram

```mermaid
erDiagram
    users ||--o{ media : owns
    users ||--o{ albums : owns
    users ||--o{ quota_reservations : has
    media ||--o{ medium_items : contains
    media ||--o{ medium_tags : has
    tags ||--o{ medium_tags : tagged_in
    albums ||--o{ media : contains
    albums ||--o{ albums : parent_of

    users {
        uuid id PK
        varchar username UK
        varchar email
        bigint quota_bytes
        bigint used_storage_bytes
        bigint reserved_storage_bytes
        timestamptz created_at
        timestamptz updated_at
    }

    quota_reservations {
        uuid id PK
        uuid user_id FK
        bigint reserved_bytes
        timestamptz created_at
        timestamptz expires_at
        varchar status
    }

    media {
        uuid id PK
        uuid user_id FK
        varchar medium_type
        varchar original_filename
        uuid leading_item_id FK
        varchar state
        jsonb metadata
        uuid album_id FK
        timestamptz created_at
        timestamptz updated_at
    }

    medium_items {
        uuid id PK
        uuid medium_id FK
        varchar item_type
        varchar storage_tier
        text storage_path
        varchar mime_type
        bigint file_size_bytes
        int width
        int height
        varchar state
        timestamptz created_at
    }

    tags {
        uuid id PK
        varchar name UK
        timestamptz created_at
    }

    medium_tags {
        uuid medium_id FK
        uuid tag_id FK
        timestamptz created_at
    }

    albums {
        uuid id PK
        uuid user_id FK
        uuid parent_id FK
        varchar title
        text description
        uuid cover_medium_id FK
        timestamptz created_at
        timestamptz updated_at
    }
```

### Key Tables

**users**
- Stores user accounts from OAuth IDP
- Tracks quota allocation and usage
- Indexed on: id, username

**media**
- Aggregate root for medium
- Stores metadata as JSONB for flexibility
- Indexed on: id, user_id, state, created_at, (metadata->>'taken_at')

**medium_items**
- Child entity within Medium aggregate
- Stores all variants (original, thumbnail, preview, etc.)
- Cascade deletes with parent medium
- Indexed on: id, medium_id, item_type, state

**tags**
- Normalized tag names
- Many-to-many with media via medium_tags

**quota_reservations**
- Temporary quota holds during upload
- Cleaned up by background job
- Indexed on: user_id, status, expires_at

---

## State Machines

### Medium State Machine

```mermaid
stateDiagram-v2
    [*] --> Uploading: create()

    Uploading --> Processing: MediumUploaded event
    note right of Uploading
        File being uploaded
        to temporary storage
    end note

    Processing --> Ready: All required processing complete
    Processing --> Failed: Processing error
    note right of Processing
        - EXIF extraction
        - Move to permanent
        - Variant generation
    end note

    Ready --> [*]: delete()
    note right of Ready
        All processing done
        Medium available
    end note

    Failed --> Processing: retry()
    Failed --> [*]: delete()
    note right of Failed
        Processing failed
        May be retryable
    end note
```

**Valid State Transitions:**
- `null → Uploading` (create)
- `Uploading → Processing` (upload complete)
- `Processing → Ready` (processing complete)
- `Processing → Failed` (error)
- `Failed → Processing` (retry)
- `Ready → [deleted]` (delete)
- `Failed → [deleted]` (delete)

### MediumItem State Machine

```mermaid
stateDiagram-v2
    [*] --> Pending: create()
    Pending --> Ready: Generation successful
    Pending --> Failed: Generation failed
    Ready --> [*]: delete()
    Failed --> Pending: retry()
    Failed --> [*]: delete()

    note right of Pending
        Variant generation
        queued or in progress
    end note

    note right of Ready
        Variant available
        for download
    end note
```

---

## Deployment Architecture

### Single Server Deployment (Current)

```mermaid
graph TB
    subgraph Server["Single Server"]
        subgraph App["Application (Port 8080)"]
            API[Axum API Server]
            Workers[Event Bus Workers]
        end

        DB[(PostgreSQL<br/>localhost:5432)]
        FileSystem[(Local Filesystem<br/>/var/photonic/storage)]
        ExifTool[exiftool binary]
    end

    Internet[Internet]
    RevProxy[Reverse Proxy<br/>nginx/Caddy]

    Internet --> RevProxy
    RevProxy --> API
    API --> DB
    API --> FileSystem
    API --> ExifTool
    Workers --> DB
    Workers --> FileSystem
    Workers --> ExifTool

    style API fill:#4A90E2,color:#fff
    style DB fill:#3498DB,color:#fff
    style FileSystem fill:#2ECC71,color:#fff
```

**Components:**
- **Reverse Proxy** - SSL termination, load balancing
- **API Server** - Axum application
- **PostgreSQL** - Local database
- **Filesystem** - Local storage tiers
- **exiftool** - System binary

### Scalable Deployment (Future)

```mermaid
graph TB
    subgraph LB["Load Balancer"]
        NGINX[nginx/HAProxy]
    end

    subgraph APIServers["API Servers (Stateless)"]
        API1[API Server 1]
        API2[API Server 2]
        API3[API Server N]
    end

    subgraph Workers["Worker Pool"]
        Worker1[Worker 1<br/>EXIF]
        Worker2[Worker 2<br/>File Moving]
        Worker3[Worker 3<br/>Variants]
    end

    NGINX --> API1
    NGINX --> API2
    NGINX --> API3

    MessageBus[(NATS/Kafka<br/>Event Bus)]

    API1 --> MessageBus
    API2 --> MessageBus
    API3 --> MessageBus

    MessageBus --> Worker1
    MessageBus --> Worker2
    MessageBus --> Worker3

    DB[(PostgreSQL<br/>Cluster)]
    S3[(S3-Compatible<br/>Object Storage)]

    API1 --> DB
    API2 --> DB
    API3 --> DB

    API1 --> S3
    API2 --> S3
    API3 --> S3

    Worker1 --> DB
    Worker2 --> DB
    Worker3 --> DB

    Worker1 --> S3
    Worker2 --> S3
    Worker3 --> S3

    style NGINX fill:#F39C12,color:#fff
    style MessageBus fill:#E74C3C,color:#fff
    style DB fill:#3498DB,color:#fff
    style S3 fill:#2ECC71,color:#fff
```

**Scaling Strategy:**
- **API Servers** - Horizontal scaling, stateless
- **Workers** - Scale based on queue depth
- **Database** - Primary-replica, read replicas
- **Storage** - S3-compatible object storage
- **Event Bus** - NATS or Kafka for distributed events

---

## Technology Stack

### Current Stack

**Backend:**
- Language: Rust (stable)
- Web Framework: Axum
- Database: PostgreSQL + SQLx
- Event Bus: In-memory (async channels)
- Authentication: JWT (jwt-authorizer)
- Tracing: OpenTelemetry + OTLP
- File Storage: Filesystem

**External:**
- EXIF: exiftool
- OAuth2: Any OIDC provider (Keycloak, Auth0, etc.)

### Future Enhancements

**Backend:**
- Event Bus: NATS or Kafka
- Storage: S3-compatible object storage
- Image Processing: libvips for better performance
- ML/AI: TensorFlow Serving or custom model server
- Cache: Redis for API caching
- Search: Elasticsearch or Meilisearch for full-text

**Frontend:**
- Web App: React or Vue.js
- Mobile: React Native or Flutter

---

## Monitoring and Observability

### Metrics to Track

```
Application Metrics:
- Request rate (req/sec)
- Response times (p50, p95, p99)
- Error rates (4xx, 5xx)
- Upload throughput (MB/sec)
- Processing pipeline latency

Business Metrics:
- Active users
- Total media count
- Storage usage by tier
- Quota utilization
- Processing success rate

Infrastructure Metrics:
- CPU usage
- Memory usage
- Disk I/O
- Database connections
- Event bus queue depth
```

### Logging Strategy

```
Structured logging with:
- Request ID (correlation)
- User ID
- Medium ID
- Timestamp
- Log level (ERROR, WARN, INFO, DEBUG)
- Component (domain, application, infrastructure)
- Message
- Context (additional fields)
```

### Tracing

```
OpenTelemetry distributed tracing:
- Trace upload request through entire pipeline
- Track processing steps (EXIF, move, variants)
- Identify bottlenecks
- Debug failures
```

---

## Summary

This architecture provides:

✅ **Clean Architecture** - Clear separation of concerns

✅ **Hexagonal Architecture** - Ports and adapters pattern

✅ **Event-Driven** - Async processing via domain events

✅ **Scalable** - Can scale horizontally

✅ **Testable** - Pure domain logic, mockable infrastructure

✅ **Flexible** - Easy to swap databases, storage, event bus

✅ **Observable** - Comprehensive logging, metrics, tracing

✅ **Resilient** - Error handling, retries, saga patterns

The architecture is designed to start simple (single server) and scale to distributed systems as needed, without major refactoring due to the clean separation of concerns.