use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, Utc};
use event_sourcing::aggregate::traits::{Aggregate, ApplyEvent};
use mime::Mime;
use serde::{Deserialize, Serialize};
use snafu::ensure;
use uuid::Uuid;

use super::{
    camera::GpsCoordinates,
    file::{Dimensions, Filename, Priority},
    storage::{FileLocation, StorageTier},
};
use crate::{
    aggregate::{AggregateRoot, AggregateVersion},
    error::{DomainResult, ValidationSnafu},
    medium::events::{MediumCreatedEvent, MediumItemCreatedEvent, MediumUpdatedEvent},
    user::UserId,
};

pub type MediumId = Uuid;
pub type MediumItemId = Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MediumItemType {
    Original,
    Edit,
    Preview,
    Sidecar,
}

impl From<Mime> for MediumType {
    fn from(value: Mime) -> Self {
        match (value.type_(), value.subtype()) {
            (mime::IMAGE, mime::SVG) => MediumType::Vector,
            (mime::IMAGE, mime::GIF) => MediumType::Gif,
            (mime::IMAGE, _) => MediumType::Photo,
            (mime::VIDEO, _) => MediumType::Video,
            _ => MediumType::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medium {
    pub id: MediumId,
    pub owner_id: UserId,
    pub medium_type: MediumType,
    pub leading_item_id: MediumItemId,
    pub taken_at: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_coordinates: Option<GpsCoordinates>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<MediumItem>,
    pub version: AggregateVersion,
}

impl Default for Medium {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            medium_type: MediumType::Other,
            leading_item_id: Uuid::nil(),
            taken_at: None,
            camera_make: None,
            camera_model: None,
            gps_coordinates: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
            items: Vec::new(),
            version: 0,
        }
    }
}

impl AggregateRoot for Medium {
    fn aggregate_type() -> &'static str {
        "Medium"
    }

    fn version(&self) -> AggregateVersion {
        self.version
    }
}

impl Aggregate for Medium {
    type Id = Uuid;

    fn aggregate_type() -> &'static str {
        "Medium"
    }
}

impl ApplyEvent<MediumCreatedEvent> for Medium {
    fn apply(&mut self, e: &MediumCreatedEvent) {
        self.id = e.medium_id;
        self.owner_id = e.user_id;
        self.medium_type = e.medium_type;
        self.leading_item_id = e.initial_item.id;
        self.items = vec![e.initial_item.clone()];
        self.version += 1;
    }
}

impl ApplyEvent<MediumItemCreatedEvent> for Medium {
    fn apply(&mut self, _e: &MediumItemCreatedEvent) {
        // Item data is added via add_item(); during replay this is a no-op
        // because the item is already constructed from the event data.
        // Full reconstruction from events will be implemented when
        // MediumItemCreatedEvent is enriched with all item fields.
        self.version += 1;
    }
}

impl ApplyEvent<MediumUpdatedEvent> for Medium {
    fn apply(&mut self, e: &MediumUpdatedEvent) {
        self.taken_at = e.taken_at.clone();
        self.camera_make = e.camera_make.clone();
        self.camera_model = e.camera_model.clone();
        self.gps_coordinates = e.gps_coordinates;
        self.version += 1;
    }
}

/// Read model for listing media - optimized for list queries without full details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumListItem {
    pub id: MediumId,
    pub owner_id: UserId,
    pub medium_type: MediumType,
    pub leading_item_id: MediumItemId,
    pub taken_at: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_coordinates: Option<GpsCoordinates>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<MediumItem>,
}

impl Medium {
    pub fn new(request: MediumCreateRequest) -> DomainResult<(Self, MediumCreatedEvent)> {
        ensure!(
            !request.medium_item.locations.is_empty(),
            ValidationSnafu {
                message: "MediumItem locations must not be empty"
            }
        );

        let id = MediumId::new_v4();
        let now = Utc::now();
        let item = MediumItem::new(id, request.medium_item);

        let mut medium = Self {
            id,
            owner_id: request.owner_id,
            medium_type: request.medium_type,
            leading_item_id: item.id,
            taken_at: request.taken_at,
            camera_make: request.camera_make,
            camera_model: request.camera_model,
            gps_coordinates: None,
            created_at: now,
            updated_at: now,
            items: vec![item.clone()],
            version: 0,
        };

        let mut event =
            MediumCreatedEvent::new(medium.id, medium.owner_id, medium.medium_type, item);
        event.metadata.expected_version = 0;
        medium.version = 1;

        Ok((medium, event))
    }

    pub fn add_item(
        &mut self,
        request: MediumItemCreateRequest,
    ) -> DomainResult<MediumItemCreatedEvent> {
        ensure!(
            !request.locations.is_empty(),
            ValidationSnafu {
                message: "MediumItem locations must not be empty"
            }
        );
        let owner_id = request.owner_id;
        let item = MediumItem::new(self.id, request);
        let event = MediumItemCreatedEvent::new(
            owner_id,
            self.id,
            item.id,
            item.medium_item_type,
            item.locations.first().unwrap().clone(),
            item.mime.clone(),
            item.filename.clone(),
            item.filesize,
            item.priority,
            item.dimensions,
        );
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(event)
    }

    pub fn find_item_mut(&mut self, item_id: MediumItemId) -> Option<&mut MediumItem> {
        self.items.iter_mut().find(|i| i.id == item_id)
    }

    /// Update basic metadata fields (denormalized from Metadata event)
    /// Called when MetadataExtractedEvent is received
    pub fn update_basic_metadata(
        &mut self,
        taken_at: Option<DateTime<FixedOffset>>,
        camera_make: Option<String>,
        camera_model: Option<String>,
        gps_coordinates: Option<GpsCoordinates>,
    ) -> MediumUpdatedEvent {
        let mut event = MediumUpdatedEvent::new(
            self.id,
            self.owner_id,
            taken_at.clone(),
            camera_make.clone(),
            camera_model.clone(),
            gps_coordinates,
        );
        event.metadata.expected_version = self.version;
        self.taken_at = taken_at;
        self.camera_make = camera_make;
        self.camera_model = camera_model;
        self.gps_coordinates = gps_coordinates;
        self.updated_at = Utc::now();
        self.version += 1;
        event
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumItem {
    pub id: MediumItemId,
    pub medium_id: MediumId,
    pub medium_item_type: MediumItemType,
    #[serde(with = "crate::serde_helpers::mime_serde")]
    pub mime: Mime,
    pub filename: Filename,
    pub filesize: Byte,
    pub priority: Priority,
    pub dimensions: Option<Dimensions>,
    pub locations: Vec<FileLocation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MediumItem {
    pub fn add_location(&mut self, location: FileLocation) {
        self.locations.push(location);
        self.updated_at = Utc::now();
    }

    pub fn remove_location(&mut self, storage_tier: StorageTier) {
        self.locations.retain(|l| l.storage_tier != storage_tier);
        self.updated_at = Utc::now();
    }

    fn new(medium_id: MediumId, request: MediumItemCreateRequest) -> Self {
        Self {
            id: MediumItemId::new_v4(),
            medium_id,
            medium_item_type: request.medium_item_type,
            mime: request.mime,
            filename: request.filename,
            filesize: request.filesize,
            priority: request.priority,
            dimensions: request.dimensions,
            locations: request.locations,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediumCreateRequest {
    pub owner_id: UserId,
    pub medium_type: MediumType,
    pub taken_at: Option<DateTime<FixedOffset>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub medium_item: MediumItemCreateRequest,
}

#[derive(Debug, Clone)]
pub struct MediumItemCreateRequest {
    pub owner_id: UserId,
    pub medium_item_type: MediumItemType,
    pub mime: Mime,
    pub filename: Filename,
    pub filesize: Byte,
    pub priority: Priority,
    pub dimensions: Option<Dimensions>,
    pub locations: Vec<FileLocation>,
}
