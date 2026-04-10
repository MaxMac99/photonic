# Event Sourcing Implementation Plan for Photonic

## Goal

Implement proper event sourcing to gain hands-on experience with ES patterns while building a real
photo management application.

## Learning Objectives

By implementing ES in Photonic, you'll gain experience with:

1. **Core ES Patterns**
    - Event-first design
    - Aggregate roots with behavior
    - Event replay and state reconstruction
    - Optimistic concurrency control

2. **CQRS Implementation**
    - Command/Query separation
    - Projections (read models)
    - Eventual consistency

3. **Domain-Driven Design**
    - Rich domain models
    - Value objects
    - Bounded contexts
    - Domain events

4. **Production ES Concepts**
    - Event versioning and upcasting
    - Snapshots for performance
    - Event bus integration
    - Projection rebuilding

## Implementation Strategy

### Phase 1: Foundation (Week 1-2)

**Goal**: Set up event store infrastructure and implement first aggregate

- [ ] Create event store schema
- [ ] Implement event store repository
- [ ] Build Album aggregate with events
- [ ] Create first projection (album list view)
- [ ] Test event replay

**Learning Focus**: Event store basics, aggregate patterns, event replay

### Phase 2: Complete Album Aggregate (Week 3-4)

**Goal**: Implement full album lifecycle with all operations

- [ ] Add all album operations (rename, delete, add/remove media)
- [ ] Implement optimistic concurrency control
- [ ] Create multiple projections for different queries
- [ ] Handle projection updates via event bus
- [ ] Add integration tests

**Learning Focus**: Domain events, projections, CQRS, eventual consistency

### Phase 3: Medium Aggregate (Week 5-6)

**Goal**: Apply ES patterns to a more complex aggregate with processing workflow

- [ ] Implement Medium aggregate with ES
- [ ] Model photo processing as events
- [ ] Handle cross-aggregate communication
- [ ] Create projections for media queries

**Learning Focus**: Complex workflows, cross-aggregate communication, process managers

### Phase 4: Advanced Patterns (Week 7-8)

**Goal**: Learn production ES patterns

- [ ] Implement snapshots for performance
- [ ] Add event versioning/upcasting
- [ ] Build projection rebuilding tool
- [ ] Add event metadata (causation, correlation)
- [ ] Implement sagas/process managers

**Learning Focus**: Snapshots, event versioning, production concerns

### Phase 5: Production Readiness (Week 9-10)

**Goal**: Make it production-ready

- [ ] Monitoring and observability
- [ ] Projection lag tracking
- [ ] Error handling and retry logic
- [ ] Event store backups
- [ ] Performance optimization

**Learning Focus**: Operations, monitoring, production concerns

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        API Layer (Axum)                          │
│  HTTP Handlers → Translate requests to commands                 │
└─────────────────────────┬───────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────────┐
│                   Application Layer (CQRS)                       │
│                                                                   │
│  Commands                           Queries                      │
│  ├─ CreateAlbumHandler              ├─ GetAlbumsQuery          │
│  ├─ AddMediumToAlbumHandler         ├─ GetAlbumDetailsQuery    │
│  └─ ...                              └─ ...                      │
│                                                                   │
│  Handlers load aggregates, execute business logic, save events  │
└─────────────────────────┬──────────────────────┬────────────────┘
                          ↓                      ↓
┌─────────────────────────────────────┐  ┌──────────────────────┐
│        Domain Layer                  │  │   Read Models        │
│                                      │  │  (Projections)       │
│  Album Aggregate                     │  │                      │
│  ├─ create()                         │  │  album_list_view     │
│  ├─ rename()                         │  │  album_details_view  │
│  ├─ add_medium()                     │  │  media_search_view   │
│  └─ [generates events]               │  │                      │
│                                      │  │  Optimized for       │
│  from_events() → Reconstruct         │  │  specific queries    │
│                                      │  │                      │
│  Medium Aggregate                    │  │                      │
│  ├─ upload()                         │  └──────────────────────┘
│  ├─ process_exif()                   │              ↑
│  └─ [generates events]               │              │
└─────────────────────────┬────────────┘              │
                          ↓                           │
┌─────────────────────────────────────────────────────┴───────────┐
│                  Infrastructure Layer                             │
│                                                                   │
│  Event Store (PostgreSQL)          Event Bus                     │
│  ├─ append_events()                ├─ publish()                  │
│  ├─ load_events()                  └─ subscribe()                │
│  └─ [event_store table]                    ↓                     │
│                                   Projection Handlers            │
│  Snapshot Store (optional)        ├─ AlbumProjectionHandler     │
│  └─ [snapshots table]              └─ MediumProjectionHandler    │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

## Database Schema

### Event Store

```sql
-- Core event store table (append-only, immutable)
CREATE TABLE event_store (
    id BIGSERIAL PRIMARY KEY,

    -- Aggregate identification
    aggregate_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL,

    -- Event data
    version BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSONB NOT NULL,

    -- Metadata for debugging and correlation
    metadata JSONB,

    -- Causation tracking (what caused this event)
    causation_id UUID,
    correlation_id UUID,

    -- Timestamp
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Ensure no duplicate versions per aggregate (optimistic locking)
    UNIQUE(aggregate_id, version)
);

-- Indexes for performance
CREATE INDEX idx_event_store_aggregate ON event_store(aggregate_id, version);
CREATE INDEX idx_event_store_type ON event_store(aggregate_type, created_at);
CREATE INDEX idx_event_store_event_type ON event_store(event_type, created_at);
CREATE INDEX idx_event_store_correlation ON event_store(correlation_id) WHERE correlation_id IS NOT NULL;

-- Snapshots table (optional, for performance)
CREATE TABLE snapshots (
    aggregate_id UUID PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    version BIGINT NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_snapshots_type ON snapshots(aggregate_type, created_at);
```

### Read Models (Projections)

```sql
-- Album list view (optimized for listing albums)
CREATE TABLE album_list_view (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,

    -- Denormalized counts for performance
    media_count INT DEFAULT 0,

    -- First and last photo dates (for sorting)
    first_photo_at TIMESTAMP,
    last_photo_at TIMESTAMP,

    -- Soft delete support
    deleted_at TIMESTAMP,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,

    -- Track which event version this projection is at
    last_event_version BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX idx_album_list_owner ON album_list_view(owner_id, updated_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_album_list_dates ON album_list_view(first_photo_at, last_photo_at) WHERE deleted_at IS NULL;

-- Album details view (optimized for single album with all details)
CREATE TABLE album_details_view (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,

    -- Array of media IDs (denormalized)
    media_ids UUID[] DEFAULT '{}',

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    last_event_version BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX idx_album_details_owner ON album_details_view(owner_id);

-- Media search view (optimized for searching media)
CREATE TABLE media_search_view (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    album_id UUID,

    -- Searchable fields
    original_filename TEXT,
    content_type TEXT,

    -- EXIF data (denormalized for search)
    camera_make TEXT,
    camera_model TEXT,
    taken_at TIMESTAMP,
    location_lat DECIMAL,
    location_lon DECIMAL,

    -- Processing status
    processing_status TEXT NOT NULL,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,

    last_event_version BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX idx_media_search_owner ON media_search_view(owner_id, taken_at DESC);
CREATE INDEX idx_media_search_album ON media_search_view(album_id, taken_at DESC);
CREATE INDEX idx_media_search_camera ON media_search_view(camera_make, camera_model);
CREATE INDEX idx_media_search_location ON media_search_view(location_lat, location_lon) WHERE location_lat IS NOT NULL;

-- Projection tracking (track projection progress)
CREATE TABLE projection_state (
    projection_name TEXT PRIMARY KEY,
    last_processed_event_id BIGINT NOT NULL,
    last_processed_at TIMESTAMP NOT NULL,
    error_count INT DEFAULT 0,
    last_error TEXT,
    last_error_at TIMESTAMP
);
```

## Core Domain Events

```rust
// src/event/album/events.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlbumEvent {
    Created(AlbumCreated),
    Renamed(AlbumRenamed),
    DescriptionChanged(AlbumDescriptionChanged),
    MediumAdded(MediumAddedToAlbum),
    MediumRemoved(MediumRemovedFromAlbum),
    Deleted(AlbumDeleted),
    Restored(AlbumRestored),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumCreated {
    pub album_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumRenamed {
    pub album_id: Uuid,
    pub old_title: String,
    pub new_title: String,
    pub renamed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDescriptionChanged {
    pub album_id: Uuid,
    pub old_description: Option<String>,
    pub new_description: Option<String>,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumAddedToAlbum {
    pub album_id: Uuid,
    pub medium_id: Uuid,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumRemovedFromAlbum {
    pub album_id: Uuid,
    pub medium_id: Uuid,
    pub removed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDeleted {
    pub album_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumRestored {
    pub album_id: Uuid,
    pub restored_at: DateTime<Utc>,
}
```

## Rich Domain Model Example

```rust
// src/event/album/aggregate.rs
use super::events::*;
use super::value_objects::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Album aggregate root
/// This is the source of truth for album state, reconstructed from events
pub struct Album {
    // Identity
    id: AlbumId,
    owner_id: UserId,

    // State
    title: AlbumTitle,
    description: Option<String>,
    media: Vec<MediumId>,

    // Metadata
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,

    // Event sourcing infrastructure
    version: u64,
    uncommitted_events: Vec<AlbumEvent>,
}

impl Album {
    /// Create a new album (generates AlbumCreated event)
    pub fn create(
        id: AlbumId,
        owner_id: UserId,
        title: AlbumTitle,
        description: Option<String>,
    ) -> Result<Self, DomainError> {
        let now = Utc::now();

        let event = AlbumEvent::Created(AlbumCreated {
            album_id: id.value(),
            owner_id: owner_id.value(),
            title: title.value().to_string(),
            description: description.clone(),
            created_at: now,
        });

        let mut album = Self::default();
        album.apply(event);

        Ok(album)
    }

    /// Rename the album (generates AlbumRenamed event)
    pub fn rename(&mut self, new_title: AlbumTitle) -> Result<(), DomainError> {
        // Business rule: Title must be different
        if self.title == new_title {
            return Err(DomainError::TitleUnchanged);
        }

        // Business rule: Cannot rename deleted album
        if self.deleted_at.is_some() {
            return Err(DomainError::AlbumDeleted);
        }

        let event = AlbumEvent::Renamed(AlbumRenamed {
            album_id: self.id.value(),
            old_title: self.title.value().to_string(),
            new_title: new_title.value().to_string(),
            renamed_at: Utc::now(),
        });

        self.apply(event);
        Ok(())
    }

    /// Add a medium to the album (generates MediumAddedToAlbum event)
    pub fn add_medium(&mut self, medium_id: MediumId) -> Result<(), DomainError> {
        // Business rule: Cannot add to deleted album
        if self.deleted_at.is_some() {
            return Err(DomainError::AlbumDeleted);
        }

        // Business rule: Cannot add duplicate media
        if self.media.contains(&medium_id) {
            return Err(DomainError::MediumAlreadyInAlbum);
        }

        // Business rule: Album size limit
        if self.media.len() >= 10000 {
            return Err(DomainError::AlbumFull);
        }

        let event = AlbumEvent::MediumAdded(MediumAddedToAlbum {
            album_id: self.id.value(),
            medium_id: medium_id.value(),
            added_at: Utc::now(),
        });

        self.apply(event);
        Ok(())
    }

    /// Remove a medium from the album (generates MediumRemovedFromAlbum event)
    pub fn remove_medium(&mut self, medium_id: MediumId) -> Result<(), DomainError> {
        // Business rule: Cannot remove from deleted album
        if self.deleted_at.is_some() {
            return Err(DomainError::AlbumDeleted);
        }

        // Business rule: Medium must be in album
        if !self.media.contains(&medium_id) {
            return Err(DomainError::MediumNotInAlbum);
        }

        let event = AlbumEvent::MediumRemoved(MediumRemovedFromAlbum {
            album_id: self.id.value(),
            medium_id: medium_id.value(),
            removed_at: Utc::now(),
        });

        self.apply(event);
        Ok(())
    }

    /// Delete the album (soft delete, generates AlbumDeleted event)
    pub fn delete(&mut self) -> Result<(), DomainError> {
        // Business rule: Cannot delete already deleted album
        if self.deleted_at.is_some() {
            return Err(DomainError::AlbumAlreadyDeleted);
        }

        let event = AlbumEvent::Deleted(AlbumDeleted {
            album_id: self.id.value(),
            deleted_at: Utc::now(),
        });

        self.apply(event);
        Ok(())
    }

    /// Restore a deleted album (generates AlbumRestored event)
    pub fn restore(&mut self) -> Result<(), DomainError> {
        // Business rule: Can only restore deleted albums
        if self.deleted_at.is_none() {
            return Err(DomainError::AlbumNotDeleted);
        }

        let event = AlbumEvent::Restored(AlbumRestored {
            album_id: self.id.value(),
            restored_at: Utc::now(),
        });

        self.apply(event);
        Ok(())
    }

    /// Reconstruct aggregate from event history (event sourcing core concept)
    pub fn from_events(events: Vec<AlbumEvent>) -> Result<Self, DomainError> {
        if events.is_empty() {
            return Err(DomainError::NoEvents);
        }

        let mut album = Self::default();

        for event in events {
            album.apply_without_tracking(event)?;
        }

        Ok(album)
    }

    /// Apply event and update state (core event sourcing pattern)
    fn apply(&mut self, event: AlbumEvent) {
        self.apply_without_tracking(event.clone())
            .expect("Event application should not fail");
        self.uncommitted_events.push(event);
    }

    /// Apply event without adding to uncommitted (for replay)
    fn apply_without_tracking(&mut self, event: AlbumEvent) -> Result<(), DomainError> {
        match event {
            AlbumEvent::Created(e) => {
                self.id = AlbumId::from(e.album_id);
                self.owner_id = UserId::from(e.owner_id);
                self.title = AlbumTitle::new(e.title)?;
                self.description = e.description;
                self.created_at = e.created_at;
                self.updated_at = e.created_at;
            }
            AlbumEvent::Renamed(e) => {
                self.title = AlbumTitle::new(e.new_title)?;
                self.updated_at = e.renamed_at;
            }
            AlbumEvent::DescriptionChanged(e) => {
                self.description = e.new_description;
                self.updated_at = e.changed_at;
            }
            AlbumEvent::MediumAdded(e) => {
                self.media.push(MediumId::from(e.medium_id));
                self.updated_at = e.added_at;
            }
            AlbumEvent::MediumRemoved(e) => {
                self.media.retain(|id| id.value() != e.medium_id);
                self.updated_at = e.removed_at;
            }
            AlbumEvent::Deleted(e) => {
                self.deleted_at = Some(e.deleted_at);
                self.updated_at = e.deleted_at;
            }
            AlbumEvent::Restored(e) => {
                self.deleted_at = None;
                self.updated_at = e.restored_at;
            }
        }

        self.version += 1;
        Ok(())
    }

    /// Take uncommitted events (for persisting)
    pub fn take_uncommitted_events(&mut self) -> Vec<AlbumEvent> {
        std::mem::take(&mut self.uncommitted_events)
    }

    // Getters
    pub fn id(&self) -> &AlbumId { &self.id }
    pub fn owner_id(&self) -> &UserId { &self.owner_id }
    pub fn title(&self) -> &AlbumTitle { &self.title }
    pub fn version(&self) -> u64 { self.version }
    pub fn is_deleted(&self) -> bool { self.deleted_at.is_some() }
}

impl Default for Album {
    fn default() -> Self {
        Self {
            id: AlbumId::from(Uuid::nil()),
            owner_id: UserId::from(Uuid::nil()),
            title: AlbumTitle::new("").unwrap(),
            description: None,
            media: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            version: 0,
            uncommitted_events: Vec::new(),
        }
    }
}
```

## Value Objects Example

```rust
// src/event/album/value_objects.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Album title value object (enforces business rules)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumTitle(String);

impl AlbumTitle {
    pub fn new(title: impl Into<String>) -> Result<Self, DomainError> {
        let title = title.into().trim().to_string();

        // Business rule: Title cannot be empty
        if title.is_empty() {
            return Err(DomainError::EmptyTitle);
        }

        // Business rule: Title length limit
        if title.len() > 255 {
            return Err(DomainError::TitleTooLong);
        }

        Ok(Self(title))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Album ID value object (type-safe ID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlbumId(Uuid);

impl AlbumId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for AlbumId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User ID value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

/// Medium ID value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediumId(Uuid);

impl MediumId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}
```

## Next Steps

Ready to start implementing? Here's what we'll build first:

### Week 1 Focus: Album Aggregate with Event Sourcing

1. **Database setup**
    - Create event_store table
    - Create first projection (album_list_view)

2. **Domain layer**
    - Album aggregate with events
    - Value objects (AlbumId, AlbumTitle, etc.)
    - Domain events

3. **Infrastructure layer**
    - Event store repository (PostgreSQL)
    - Event serialization/deserialization

4. **Application layer**
    - CreateAlbumCommand handler
    - GetAlbumsQuery handler
    - Event bus wiring

5. **Tests**
    - Unit tests for aggregate behavior
    - Integration tests for event store
    - Test event replay

Would you like me to start implementing this? I can begin with:

1. The database migrations for event store
2. The Album aggregate with rich domain model
3. The event store repository
4. The command handlers

Let me know and I'll help you build this step by step!