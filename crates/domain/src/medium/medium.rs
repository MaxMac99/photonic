use byte_unit::Byte;
use chrono::{DateTime, FixedOffset, Utc};
use mime::Mime;
use snafu::ensure;
use uuid::Uuid;

use super::{
    camera::GpsCoordinates,
    file::{Dimensions, Filename, Priority},
    storage::{FileLocation, StorageTier},
};
use crate::{
    error::{DomainResult, ValidationSnafu},
    medium::events::{MediumCreatedEvent, MediumItemCreatedEvent, MediumUpdatedEvent},
    user::UserId,
};

pub type MediumId = Uuid;
pub type MediumItemId = Uuid;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MediumType {
    Photo,
    Video,
    LivePhoto,
    Vector,
    Sequence,
    Gif,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone)]
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
}

/// Read model for listing media - optimized for list queries without full details
#[derive(Debug, Clone)]
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
    pub fn new(
        request: MediumCreateRequest,
    ) -> DomainResult<(Self, MediumCreatedEvent, MediumItemCreatedEvent)> {
        ensure!(
            !request.medium_item.locations.is_empty(),
            ValidationSnafu {
                message: "MediumItem locations must not be empty"
            }
        );

        let id = MediumId::new_v4();
        let now = Utc::now();

        let mut medium = Self {
            id,
            owner_id: request.owner_id,
            medium_type: request.medium_type,
            leading_item_id: MediumItemId::new_v4(),
            taken_at: request.taken_at,
            camera_make: request.camera_make,
            camera_model: request.camera_model,
            gps_coordinates: None,
            created_at: now,
            updated_at: now,
            items: vec![],
        };

        let item_created_event = medium.add_item(request.medium_item)?;
        medium.leading_item_id = medium.items[0].id;

        let medium_created_event = MediumCreatedEvent::new(
            medium.id,
            medium.owner_id,
            medium.medium_type,
            medium.leading_item_id,
            medium.items[0].locations.first().unwrap().clone(),
        );

        Ok((medium, medium_created_event, item_created_event))
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
            self.id,
            item.id,
            owner_id,
            item.medium_item_type,
            item.locations.first().unwrap().clone(),
            item.mime.clone(),
        );
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(event)
    }

    pub fn find_item_mut(&mut self, item_id: MediumItemId) -> Option<&mut MediumItem> {
        self.items.iter_mut().find(|i| i.id == item_id)
    }

    /// Update basic metadata fields (denormalized from Metadata domain)
    /// Called when MetadataExtractedEvent is received
    pub fn update_basic_metadata(
        &mut self,
        taken_at: Option<DateTime<FixedOffset>>,
        camera_make: Option<String>,
        camera_model: Option<String>,
        gps_coordinates: Option<GpsCoordinates>,
    ) -> MediumUpdatedEvent {
        self.taken_at = taken_at;
        self.camera_make = camera_make;
        self.camera_model = camera_model;
        self.gps_coordinates = gps_coordinates;
        self.updated_at = Utc::now();
        MediumUpdatedEvent::new(self.id, self.owner_id)
    }
}

#[derive(Debug, Clone)]
pub struct MediumItem {
    pub id: MediumItemId,
    pub medium_id: MediumId,
    pub medium_item_type: MediumItemType,
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
