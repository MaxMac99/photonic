# Photonic Domain Model (DDD)

This document describes the domain model for Photonic using Domain-Driven Design (DDD) principles.
The system is organized into three main bounded contexts: **User**, **Medium**, and **Album**.

## Table of Contents

- [Overview](#overview)
- [Bounded Contexts](#bounded-contexts)
- [User Context](#user-context)
- [Medium Context](#medium-context)
- [Album Context](#album-context-future)
- [Cross-Context Relationships](#cross-context-relationships)
- [Domain Events](#domain-events)

---

## Overview

### DDD Principles Applied

1. **Bounded Contexts** - Clear boundaries between User, Medium, and Album domains
2. **Aggregates** - User, Medium (with MediumItems), Album are aggregate roots
3. **Value Objects** - Immutable objects for concepts like Email, StorageLocation, Metadata
4. **Domain Events** - Event-driven architecture for async processing
5. **Repository Pattern** - Abstract data access behind interfaces
6. **Domain Services** - Business logic that doesn't belong to entities (QuotaService,
   StoragePathService)

### Architecture Layers

```
┌─────────────────────────────────────────────┐
│         API Layer (Axum Handlers)           │
│         Infrastructure adapters             │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│        Application Layer (CQRS)             │
│    Commands & Queries & Event Listeners     │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           Domain Layer (PURE)               │
│  Entities, Value Objects, Domain Services   │
│    Events, Repository Ports (interfaces)    │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│        Infrastructure Layer                 │
│   Repository Implementations, Event Bus     │
│   Storage, External Services (exiftool)     │
└─────────────────────────────────────────────┘
```

**Key Rule:** Domain layer has ZERO dependencies on infrastructure or frameworks. All external
dependencies are abstracted through ports (interfaces).

---

## Bounded Contexts

### Context Map

```
┌──────────────┐          ┌──────────────┐          ┌──────────────┐
│              │          │              │          │              │
│     User     │◄────────►│    Medium    │◄────────►│    Album     │
│   Context    │  user_id │   Context    │ album_id │   Context    │
│              │          │              │          │  (Future)    │
└──────────────┘          └──────────────┘          └──────────────┘
       │                         │                         │
       │                         │                         │
   Manages                   Stores                   Organizes
   quota &                   media &                  media into
   auth                      variants                 collections
```

**Relationships:**

- User Context → Medium Context: One user has many media (one-to-many)
- Medium Context → Album Context: One medium belongs to one album (many-to-one, optional)
- Album Context → User Context: One user has many albums (one-to-many)

**Communication:**

- Contexts communicate via domain events
- Use IDs (UUIDs) to reference entities across contexts
- No direct entity references across contexts

---

## User Context

**Purpose:** Authentication, authorization, and storage quota management

### Aggregate: User

```rust
// Domain Entity
pub struct User {
    id: UserId,
    username: Username,
    email: Option<Email>,
    quota: Quota,
    used_storage: StorageSize,
    reserved_storage: StorageSize,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

**Invariants:**

- Username must be unique across system
- `used_storage + reserved_storage <= quota` (enforced by QuotaService)
- Username cannot be empty or contain invalid characters
- Email must be valid format (if provided)

**Behaviors:**

- `update_username(new_username: Username) -> Result<()>`
- `update_email(new_email: Option<Email>) -> Result<()>`
- `update_quota(new_quota: Quota) -> Result<()>`
- `get_available_quota() -> StorageSize`

---

### Value Objects

#### UserId

```rust
pub struct UserId(Uuid);
```

- Immutable identifier
- Generated once on user creation
- Used as foreign key in other contexts

#### Username

```rust
pub struct Username(String);

impl Username {
    pub fn new(value: String) -> Result<Self> {
        // Validation:
        // - Length: 3-50 characters
        // - Allowed: alphanumeric, underscore, hyphen
        // - No leading/trailing whitespace
        // - Not reserved words (admin, system, etc.)
    }
}
```

**Validation Rules:**

- Length: 3-50 characters
- Pattern: `^[a-zA-Z0-9_-]+$`
- Case-insensitive uniqueness
- No profanity or reserved words

#### Email

```rust
pub struct Email(String);

impl Email {
    pub fn new(value: String) -> Result<Self> {
        // Validation: RFC 5322 email format
        // Use email_address crate for validation
    }
}
```

#### Quota

```rust
pub struct Quota(u64); // bytes

impl Quota {
    pub fn from_bytes(bytes: u64) -> Self
    pub fn from_gb(gb: u32) -> Self
    pub fn as_bytes(&self) -> u64
    pub fn as_gb(&self) -> f64
}
```

**Examples:**

- 10 GB = 10,737,418,240 bytes
- 50 GB = 53,687,091,200 bytes
- Unlimited = u64::MAX (special value)

#### StorageSize

```rust
pub struct StorageSize(u64); // bytes

impl StorageSize {
    pub fn zero() -> Self
    pub fn from_bytes(bytes: u64) -> Self
    pub fn add(&self, other: StorageSize) -> Self
    pub fn subtract(&self, other: StorageSize) -> Result<Self>
}
```

#### QuotaReservation

```rust
pub struct QuotaReservation {
    id: ReservationId,
    user_id: UserId,
    reserved_bytes: u64,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    status: ReservationStatus,
}

pub enum ReservationStatus {
    Active,
    Committed,
    Released,
}
```

---

### Domain Services

#### QuotaService

```rust
pub struct QuotaService {
    user_repository: Arc<dyn UserRepository>,
    reservation_repository: Arc<dyn ReservationRepository>,
}

impl QuotaService {
    /// Reserve quota for upcoming upload
    pub async fn reserve_quota(
        &self,
        user_id: UserId,
        size: u64
    ) -> Result<QuotaReservation>;

    /// Commit reservation after successful upload
    pub async fn commit_reservation(
        &self,
        reservation_id: ReservationId
    ) -> Result<()>;

    /// Release reservation on failure
    pub async fn release_reservation(
        &self,
        reservation_id: ReservationId
    ) -> Result<()>;

    /// Check available quota
    pub async fn check_available_quota(
        &self,
        user_id: UserId
    ) -> Result<StorageSize>;

    /// Cleanup expired reservations (background job)
    pub async fn cleanup_expired_reservations(&self) -> Result<usize>;
}
```

**Business Rules:**

- Reservations expire after 1 hour
- Available = quota - used_storage - reserved_storage
- Atomic updates to prevent race conditions
- Idempotent operations (commit/release can be called multiple times)

---

### Repository Ports (Interfaces)

#### UserRepository

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>>;
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn update_storage_usage(&self, user_id: UserId, delta: i64) -> Result<()>;
    async fn ensure_exists(&self, user_input: UserInput) -> Result<User>;
}
```

#### ReservationRepository

```rust
#[async_trait]
pub trait ReservationRepository: Send + Sync {
    async fn create(&self, reservation: &QuotaReservation) -> Result<()>;
    async fn find_by_id(&self, id: ReservationId) -> Result<Option<QuotaReservation>>;
    async fn update_status(&self, id: ReservationId, status: ReservationStatus) -> Result<()>;
    async fn find_expired(&self) -> Result<Vec<QuotaReservation>>;
    async fn delete(&self, id: ReservationId) -> Result<()>;
}
```

---

### Domain Events

```rust
pub struct UserCreated {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub username: Username,
    pub email: Option<Email>,
    pub quota_bytes: u64,
    pub occurred_at: DateTime<Utc>,
}

pub struct UserQuotaUpdated {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub old_quota: Quota,
    pub new_quota: Quota,
    pub occurred_at: DateTime<Utc>,
}

pub struct QuotaReserved {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub reservation_id: ReservationId,
    pub reserved_bytes: u64,
    pub available_after_reservation: u64,
    pub occurred_at: DateTime<Utc>,
}

pub struct QuotaCommitted {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub reservation_id: ReservationId,
    pub committed_bytes: u64,
    pub total_used_bytes: u64,
    pub occurred_at: DateTime<Utc>,
}

pub struct QuotaReleased {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub reservation_id: ReservationId,
    pub released_bytes: u64,
    pub reason: String,
    pub occurred_at: DateTime<Utc>,
}

pub struct UserQuotaExceeded {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub requested_bytes: u64,
    pub available_bytes: u64,
    pub occurred_at: DateTime<Utc>,
}
```

---

## Medium Context

**Purpose:** Core media management, storage, and variant generation

### Aggregate: Medium

```rust
// Aggregate Root
pub struct Medium {
    // Identity
    id: MediumId,
    user_id: UserId, // Reference to User context

    // Basic Info
    medium_type: MediumType,
    original_filename: FileName,
    leading_item_id: Option<MediumItemId>,

    // State
    state: MediumState,

    // Metadata
    metadata: MediumMetadata,

    // Organization
    tags: Vec<Tag>,
    album_id: Option<AlbumId>, // Reference to Album context

    // Child Entities (part of aggregate)
    items: Vec<MediumItem>,

    // Audit
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

**Invariants:**

- Medium must have at least one MediumItem when state is "Ready"
- `leading_item_id` must reference an existing item in `items`
- `leading_item_id` should point to "Original" type item
- State transitions must be valid (see state machine)
- Tags must be unique within medium

**Behaviors:**

- `add_item(item: MediumItem) -> Result<()>`
- `set_leading_item(item_id: MediumItemId) -> Result<()>`
- `add_tags(tags: Vec<Tag>) -> Result<()>`
- `remove_tags(tags: Vec<Tag>) -> Result<()>`
- `assign_to_album(album_id: AlbumId) -> Result<()>`
- `update_state(new_state: MediumState) -> Result<()>`
- `update_metadata(metadata: MediumMetadata) -> Result<()>`

---

### Child Entity: MediumItem

```rust
// Child entity within Medium aggregate
pub struct MediumItem {
    id: MediumItemId,
    medium_id: MediumId, // Parent reference
    item_type: MediumItemType,
    storage_location: StorageLocation,
    mime_type: MimeType,
    file_size: FileSize,
    dimensions: Option<Dimensions>,
    state: ItemState,
    created_at: DateTime<Utc>,
}
```

**Invariants:**

- Item cannot exist without parent Medium
- Storage location must be valid for item type
- Mime type must match file content
- File size must be > 0

**Notes:**

- MediumItem is NOT a separate aggregate root
- Items are always loaded and saved with their parent Medium
- No direct repository for MediumItem (accessed through Medium)

---

### Value Objects

#### MediumId / MediumItemId

```rust
pub struct MediumId(Uuid);
pub struct MediumItemId(Uuid);
```

#### MediumType

```rust
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Other,
}

impl From<mime::Mime> for MediumType {
    fn from(mime: mime::Mime) -> Self {
        match (mime.type_(), mime.subtype()) {
            (mime::IMAGE, _) => MediumType::Photo,
            (mime::VIDEO, _) => MediumType::Video,
            _ => MediumType::Other,
        }
    }
}
```

#### MediumItemType

```rust
pub enum MediumItemType {
    Original,   // Original uploaded file
    Thumbnail,  // Small thumbnail (e.g., 200x200)
    Preview,    // Medium size for web display (e.g., 1024px)
    LowRes,     // Low-res JPEG for RAW files (e.g., 2048px)
    Edited,     // User-edited version
}
```

#### MediumState

```rust
pub enum MediumState {
    Uploading,   // Initial upload in progress
    Processing,  // Async task pipeline running
    Ready,       // All task complete, available
    Failed,      // Processing failed
}
```

**State Machine:**

```
Uploading ──MediumUploaded──> Processing
                                   │
                    ┌──────────────┴──────────────┐
                    │                             │
            MediumReady                MediumProcessingFailed
                    │                             │
                  Ready                        Failed
                    │                             │
              MediumDeleted                 MediumDeleted
```

#### ItemState

```rust
pub enum ItemState {
    Pending,  // Variant generation queued
    Ready,    // Variant available
    Failed,   // Generation failed
}
```

#### FileName

```rust
pub struct FileName(String);

impl FileName {
    pub fn new(value: String) -> Result<Self> {
        // Validation:
        // - Not empty
        // - Max 255 characters
        // - No path separators
        // - Sanitize for filesystem safety
    }

    pub fn sanitized(&self) -> String {
        // Remove or replace unsafe characters
        // Normalize unicode
    }

    pub fn extension(&self) -> Option<&str> {
        // Extract file extension
    }
}
```

#### StorageLocation

```rust
pub struct StorageLocation {
    tier: StorageTier,
    path: StoragePath,
}

pub enum StorageTier {
    Temporary,  // Fast storage for uploads (SSD)
    Permanent,  // Main storage for originals (HDD/S3)
    Cache,      // Generated variants (SSD/fast storage)
    Archive,    // Cold storage (future)
}

pub struct StoragePath(PathBuf);

impl StorageLocation {
    pub fn new(tier: StorageTier, path: PathBuf) -> Self
    pub fn full_path(&self, base_path: &Path) -> PathBuf
}
```

**Examples:**

```rust
// Temporary upload
StorageLocation {
    tier: Temporary,
    path: "550e8400-e29b-41d4-a716-446655440000.jpg"
}
// → /storage/temporary/550e8400-e29b-41d4-a716-446655440000.jpg

// Permanent storage
StorageLocation {
    tier: Permanent,
    path: "2024/12/Canon/IMG_1234.jpg"
}
// → /storage/permanent/2024/12/Canon/IMG_1234.jpg

// Cached thumbnail
StorageLocation {
    tier: Cache,
    path: "550e8400-e29b-41d4-a716-446655440000_thumb.jpg"
}
// → /storage/cache/550e8400-e29b-41d4-a716-446655440000_thumb.jpg
```

#### MimeType

```rust
pub struct MimeType(mime::Mime);

impl MimeType {
    pub fn from_extension(ext: &str) -> Option<Self>
    pub fn is_image(&self) -> bool
    pub fn is_video(&self) -> bool
    pub fn is_raw(&self) -> bool
}
```

**Common MIME Types:**

- JPEG: `image/jpeg`
- PNG: `image/png`
- HEIC: `image/heic`
- RAW formats: `image/x-canon-cr2`, `image/x-nikon-nef`, `image/x-sony-arw`, `image/x-adobe-dng`
- Video: `video/mp4`, `video/quicktime`

#### FileSize

```rust
pub struct FileSize(u64); // bytes

impl FileSize {
    pub fn from_bytes(bytes: u64) -> Self
    pub fn as_bytes(&self) -> u64
    pub fn as_kb(&self) -> f64
    pub fn as_mb(&self) -> f64
    pub fn as_gb(&self) -> f64
}
```

#### Dimensions

```rust
pub struct Dimensions {
    width: u32,
    height: u32,
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        // Validation: both must be > 0
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn megapixels(&self) -> f64 {
        (self.width * self.height) as f64 / 1_000_000.0
    }
}
```

#### Tag

```rust
pub struct Tag(String);

impl Tag {
    pub fn new(value: String) -> Result<Self> {
        // Validation:
        // - Lowercase
        // - Trimmed
        // - Length: 1-50 characters
        // - Pattern: ^[a-z0-9_-]+$
    }
}
```

#### MediumMetadata

```rust
pub struct MediumMetadata {
    // Date/Time
    pub taken_at: Option<DateTime<Utc>>,

    // Camera
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,

    // Exposure
    pub iso: Option<u32>,
    pub aperture: Option<f32>,         // F-number (e.g., 2.8)
    pub shutter_speed: Option<String>, // e.g., "1/250"
    pub focal_length: Option<f32>,     // in mm

    // Location
    pub gps_coordinates: Option<GpsCoordinates>,

    // Recognition (future)
    pub faces: Option<Vec<FaceDetection>>,
    pub objects: Option<Vec<ObjectLabel>>,
    pub scene: Option<SceneClassification>,
}

pub struct GpsCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

// Future: Image recognition
pub struct FaceDetection {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
}

pub struct ObjectLabel {
    pub label: String,
    pub confidence: f32,
}

pub struct SceneClassification {
    pub label: String, // "landscape", "portrait", "indoor", etc.
    pub confidence: f32,
}
```

---

### Domain Services

#### StoragePathService

```rust
pub struct StoragePathService {
    pattern: PathPattern,
}

impl StoragePathService {
    pub fn new(pattern: String) -> Result<Self>;

    /// Calculate storage path from pattern and metadata
    pub fn calculate_path(
        &self,
        metadata: &MediumMetadata,
        filename: &FileName
    ) -> Result<StoragePath>;
}
```

**Path Pattern Variables:**

- `{year}` - 4-digit year from `taken_at` (fallback: upload year)
- `{month}` - 2-digit month (01-12) from `taken_at`
- `{day}` - 2-digit day (01-31) from `taken_at`
- `{camera_make}` - Camera manufacturer (sanitized)
- `{camera_model}` - Camera model (sanitized)
- `{lens_model}` - Lens model (sanitized)
- `{filename}` - Original filename (sanitized)
- `{extension}` - File extension (jpg, cr2, etc.)
- `{user_id}` - User UUID

**Example Patterns:**

```rust
// Pattern: "{year}/{month}/{camera_make}/{filename}"
// Result: "2024/12/Canon/IMG_1234.jpg"

// Pattern: "{user_id}/{year}/{month}/{filename}"
// Result: "550e8400-e29b-41d4-a716-446655440000/2024/12/IMG_1234.jpg"

// Pattern: "{camera_make}/{camera_model}/{year}-{month}/{filename}"
// Result: "Canon/EOS_R5/2024-12/IMG_1234.jpg"
```

**Fallback Behavior:**

```rust
// If metadata field missing:
- Date fields → Use upload timestamp
- Camera fields → Use "Unknown"
- Filename → Use medium_id
```

---

### Repository Ports (Interfaces)

#### MediumRepository

```rust
#[async_trait]
pub trait MediumRepository: Send + Sync {
    // Basic CRUD
    async fn create(&self, create: MediumCreate, user_id: UserId) -> Result<Medium>;
    async fn find_by_id(&self, id: MediumId, user_id: UserId) -> Result<Option<Medium>>;
    async fn update(&self, medium: &Medium) -> Result<()>;
    async fn delete(&self, id: MediumId, user_id: UserId) -> Result<()>;

    // Queries
    async fn list_by_user(
        &self,
        user_id: UserId,
        filters: MediumFilters,
        pagination: Pagination
    ) -> Result<Vec<Medium>>;

    async fn search(
        &self,
        user_id: UserId,
        query: &str,
        filters: MediumFilters,
        pagination: Pagination
    ) -> Result<Vec<Medium>>;

    async fn count_by_user(&self, user_id: UserId, filters: MediumFilters) -> Result<usize>;
}

pub struct MediumFilters {
    pub medium_type: Option<MediumType>,
    pub album_id: Option<AlbumId>,
    pub tags: Option<Vec<Tag>>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub state: Option<MediumState>,
}

pub struct Pagination {
    pub page: usize,
    pub limit: usize,
    pub sort_by: SortField,
    pub sort_order: SortOrder,
}
```

---

### Storage Ports (Interfaces)

#### FileStorage

```rust
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Store file from stream
    async fn store_file_stream(
        &self,
        location: &StorageLocation,
        stream: Box<dyn AsyncRead + Send + Unpin>
    ) -> Result<()>;

    /// Move file between locations (within same tier or across tiers)
    async fn move_file(
        &self,
        from: &StorageLocation,
        to: &StorageLocation
    ) -> Result<()>;

    /// Delete file
    async fn delete_file(&self, location: &StorageLocation) -> Result<()>;

    /// Get file as stream
    async fn get_file_stream(
        &self,
        location: &StorageLocation
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>>;

    /// Check if file exists
    async fn file_exists(&self, location: &StorageLocation) -> Result<bool>;

    /// Get file size
    async fn get_file_size(&self, location: &StorageLocation) -> Result<u64>;
}
```

**Implementations:**

- `FilesystemStorage` - Local filesystem (current)
- `S3Storage` - AWS S3 or compatible (future)
- `HybridStorage` - Combination based on tier (future)

#### MetadataExtractor

```rust
#[async_trait]
pub trait MetadataExtractor: Send + Sync {
    /// Extract EXIF metadata from file
    async fn extract_metadata(
        &self,
        location: &StorageLocation
    ) -> Result<MediumMetadata>;
}
```

**Implementation:**

- `ExiftoolExtractor` - Uses exiftool external process

---

### Domain Events

```rust
pub struct MediumUploaded {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub file_location: StorageLocation,
    pub mime_type: MimeType,
    pub file_size: u64,
    pub original_filename: FileName,
    pub occurred_at: DateTime<Utc>,
}

pub struct MediumMetadataExtracted {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub metadata: MediumMetadata,
    pub occurred_at: DateTime<Utc>,
}

pub struct MediumMovedToFinal {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub item_id: MediumItemId,
    pub from_location: StorageLocation,
    pub to_location: StorageLocation,
    pub occurred_at: DateTime<Utc>,
}

pub struct ThumbnailCreated {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub item_id: MediumItemId,
    pub storage_location: StorageLocation,
    pub mime_type: MimeType,
    pub file_size: u64,
    pub dimensions: Dimensions,
    pub occurred_at: DateTime<Utc>,
}

pub struct PreviewCreated {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub item_id: MediumItemId,
    pub storage_location: StorageLocation,
    pub mime_type: MimeType,
    pub file_size: u64,
    pub dimensions: Dimensions,
    pub occurred_at: DateTime<Utc>,
}

pub struct LowResCreated {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub item_id: MediumItemId,
    pub storage_location: StorageLocation,
    pub mime_type: MimeType,
    pub file_size: u64,
    pub dimensions: Dimensions,
    pub occurred_at: DateTime<Utc>,
}

pub struct MediumReady {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub total_processing_time_ms: u64,
    pub variants_created: Vec<MediumItemType>,
    pub occurred_at: DateTime<Utc>,
}

pub struct MediumProcessingFailed {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub step: ProcessingStep,
    pub error: String,
    pub is_retryable: bool,
    pub occurred_at: DateTime<Utc>,
}

pub enum ProcessingStep {
    Upload,
    ExifExtraction,
    PathCalculation,
    MovingToFinal,
    ThumbnailGeneration,
    PreviewGeneration,
    LowResGeneration,
    ImageRecognition,
}

pub struct MediumDeleted {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub freed_storage_bytes: u64,
    pub occurred_at: DateTime<Utc>,
}

pub struct MediumTagged {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub added_tags: Vec<Tag>,
    pub removed_tags: Vec<Tag>,
    pub occurred_at: DateTime<Utc>,
}

pub struct MediumMovedToAlbum {
    pub event_id: Uuid,
    pub medium_id: MediumId,
    pub user_id: UserId,
    pub album_id: AlbumId,
    pub occurred_at: DateTime<Utc>,
}
```

---

## Album Context (Future)

**Purpose:** Organize media into collections and hierarchical structures

### Aggregate: Album

```rust
pub struct Album {
    id: AlbumId,
    user_id: UserId,
    parent_id: Option<AlbumId>, // For nested albums
    title: AlbumTitle,
    description: Option<String>,
    cover_medium_id: Option<MediumId>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

**Invariants:**

- Title must be unique within user's albums (same level)
- Parent album must belong to same user
- No circular references (album cannot be parent of itself)
- Cover medium must belong to user

---

### Value Objects

#### AlbumId

```rust
pub struct AlbumId(Uuid);
```

#### AlbumTitle

```rust
pub struct AlbumTitle(String);

impl AlbumTitle {
    pub fn new(value: String) -> Result<Self> {
        // Validation:
        // - Length: 1-100 characters
        // - Not empty
        // - Trimmed
    }
}
```

---

### Repository Ports

#### AlbumRepository

```rust
#[async_trait]
pub trait AlbumRepository: Send + Sync {
    async fn create(&self, album: &Album) -> Result<()>;
    async fn find_by_id(&self, id: AlbumId, user_id: UserId) -> Result<Option<Album>>;
    async fn update(&self, album: &Album) -> Result<()>;
    async fn delete(&self, id: AlbumId, user_id: UserId) -> Result<()>;
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Album>>;
    async fn list_children(&self, album_id: AlbumId) -> Result<Vec<Album>>;
}
```

---

### Domain Events

```rust
pub struct AlbumCreated {
    pub event_id: Uuid,
    pub album_id: AlbumId,
    pub user_id: UserId,
    pub title: AlbumTitle,
    pub parent_id: Option<AlbumId>,
    pub occurred_at: DateTime<Utc>,
}

pub struct AlbumUpdated {
    pub event_id: Uuid,
    pub album_id: AlbumId,
    pub user_id: UserId,
    pub occurred_at: DateTime<Utc>,
}

pub struct AlbumDeleted {
    pub event_id: Uuid,
    pub album_id: AlbumId,
    pub user_id: UserId,
    pub occurred_at: DateTime<Utc>,
}
```

---

## Cross-Context Relationships

### Reference by ID Pattern

Contexts reference entities in other contexts by ID only (not full entity):

```rust
// In Medium Context
pub struct Medium {
    user_id: UserId,    // Reference to User context
    album_id: Option<AlbumId>, // Reference to Album context
    // ...
}

// NOT allowed:
pub struct Medium {
    user: User,         // ❌ Don't embed entities from other contexts
    album: Option<Album>, // ❌ Don't embed entities from other contexts
}
```

### Event-Based Communication

Contexts communicate via domain events published to event bus:

```rust
// User Context publishes
UserCreated → subscribed by analytics, notifications

// Medium Context publishes
MediumUploaded → subscribed by EXIF extractor
MediumReady → subscribed by notification service, analytics

// Album Context publishes
AlbumCreated → subscribed by analytics
```

---

## Domain Events

### Event Base Trait

```rust
pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> Uuid;
    fn aggregate_id(&self) -> Uuid;
    fn aggregate_type(&self) -> &str;
    fn event_type(&self) -> &str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn user_id(&self) -> Option<Uuid>;
}
```

### Event Bus

```rust
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish event to all subscribers
    async fn publish<E: DomainEvent>(&self, event: E) -> Result<()>;

    /// Subscribe to event type
    fn subscribe<E: DomainEvent>(
        &self,
        handler: Arc<dyn EventHandler<E>>
    );
}

#[async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &E) -> Result<()>;
}
```

**Implementations:**

- `InMemoryEventBus` - Current implementation
- `NatsEventBus` - NATS-based (future)
- `KafkaEventBus` - Kafka-based (future)

---

## Summary

This domain model provides:

✅ **Clear Bounded Contexts** - User, Medium, Album with well-defined boundaries

✅ **Aggregate Roots** - User, Medium (with MediumItems), Album control consistency

✅ **Rich Domain Models** - Entities with behaviors, not just data containers

✅ **Value Objects** - Immutable, validated concepts like Email, StorageLocation, Metadata

✅ **Domain Services** - Business logic that spans entities (QuotaService, StoragePathService)

✅ **Port/Adapter Pattern** - Infrastructure abstracted behind interfaces

✅ **Domain Events** - Event-driven architecture for async processing

✅ **Invariants** - Business rules enforced at domain level

✅ **Zero Infrastructure Dependencies** - Pure domain logic, no framework coupling

This design enables:

- Testable domain logic (pure functions, no infrastructure)
- Flexible infrastructure (can swap databases, storage, etc.)
- Clear boundaries and responsibilities
- Event-driven async processing
- Scalable architecture