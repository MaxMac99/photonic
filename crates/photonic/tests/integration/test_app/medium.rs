use std::time::Duration;

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use photonic::domain::user::User;
use photonic_client::{
    types::{MediumDetailResponse, MediumMetadataDto, MediumTypeDto, StorageTierDto},
    Error, ResponseValue,
};
use reqwest::{header, header::HeaderValue};
use tracing::info;
use uuid::Uuid;

use crate::integration::{
    common::{
        fixtures::ImageFixture,
        polling::{poll_until, PollingConfig},
    },
    test_app::TestApp,
};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into, strip_option))]
pub struct CreateMediumRequest {
    pub album_id: Option<Uuid>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_taken: Option<DateTime<Utc>>,
    pub filename: String,
    pub medium_type: Option<MediumTypeDto>,
    pub priority: Option<i32>,
    pub tags: Vec<String>,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl From<ImageFixture> for CreateMediumRequest {
    fn from(fixture: ImageFixture) -> Self {
        let content_type = match fixture
            .filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "heic" => "image/heic",
            "dng" => "image/dng",
            _ => "application/octet-stream",
        }
        .to_string();
        CreateMediumRequest {
            album_id: None,
            camera_make: None,
            camera_model: None,
            date_taken: None,
            filename: fixture.filename.to_string(),
            medium_type: None,
            priority: None,
            tags: vec![],
            content_type,
            body: fixture.data,
        }
    }
}

impl TestApp {
    pub async fn create_medium(
        &self,
        user: &User,
        request: CreateMediumRequest,
    ) -> Result<ResponseValue<Uuid>, Error> {
        let url = format!("{}/api/v1/medium", self.base_url);

        let mut query = Vec::with_capacity(8usize);
        if let Some(v) = &request.album_id {
            query.push(("album_id", v.to_string()));
        }
        if let Some(v) = &request.camera_make {
            query.push(("camera_make", v.to_string()));
        }
        if let Some(v) = &request.camera_model {
            query.push(("camera_model", v.to_string()));
        }
        if let Some(v) = &request.date_taken {
            query.push(("date_taken", v.to_string()));
        }
        query.push(("filename", request.filename.to_string()));
        if let Some(v) = &request.medium_type {
            query.push(("medium_type", v.to_string()));
        }
        if let Some(v) = &request.priority {
            query.push(("priority", v.to_string()));
        }
        if !request.tags.is_empty() {
            query.push(("tags", request.tags.join(",")));
        }

        let client = self.client_with_user(user);
        let client = client.client();
        let request = client
            .post(url)
            .header(header::ACCEPT, HeaderValue::from_static("application/json"))
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&request.content_type).unwrap(),
            )
            .body(request.body)
            .query(&query)
            .build()?;
        let result = client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            201u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }

    /// Wait for medium to be enriched with metadata (denormalized fields on Medium entity)
    pub async fn wait_for_medium_enrichment(
        &self,
        user: &User,
        medium_id: &Uuid,
    ) -> Result<MediumDetailResponse, String> {
        poll_until(
            || async {
                let response = self.client_with_user(user).get_medium(medium_id).await;
                info!("Got response with metadata: {:?}", response);
                response.ok().and_then(|medium| {
                    Some(medium.into_inner()).filter(|m| {
                        m.camera_make.is_some() || m.camera_model.is_some() || m.taken_at.is_some()
                    })
                })
            },
            PollingConfig::new(format!("metadata extraction for medium {}", medium_id))
                .with_interval(Duration::from_millis(50))
                .with_exponential_backoff(2.0, Duration::from_secs(2)),
        )
        .await
    }

    /// Wait for metadata to be available via the dedicated metadata endpoint
    pub async fn wait_for_metadata(
        &self,
        user: &User,
        medium_id: &Uuid,
    ) -> Result<MediumMetadataDto, String> {
        poll_until(
            || async {
                let response = self
                    .client_with_user(user)
                    .get_medium_metadata(medium_id)
                    .await;
                info!("Got metadata endpoint response: {:?}", response);
                response.ok().map(|r| r.into_inner())
            },
            PollingConfig::new(format!("metadata for medium {}", medium_id))
                .with_interval(Duration::from_millis(50))
                .with_exponential_backoff(2.0, Duration::from_secs(2)),
        )
        .await
    }

    pub async fn wait_for_permanent_storage(
        &self,
        user: &User,
        medium_id: &Uuid,
    ) -> Result<MediumDetailResponse, String> {
        poll_until(
            || async {
                let response = self.client_with_user(user).get_medium(medium_id).await;
                info!(
                    "Got response checking for permanent storage: {:?}",
                    response
                );
                response.ok().and_then(|medium| {
                    let medium = medium.into_inner();
                    // Check if the primary item is in permanent storage
                    let primary_in_permanent = medium.items.iter().any(|item| {
                        item.is_primary
                            && item
                                .locations
                                .iter()
                                .any(|loc| matches!(loc.storage_tier, StorageTierDto::Permanent))
                    });
                    if primary_in_permanent {
                        Some(medium)
                    } else {
                        None
                    }
                })
            },
            PollingConfig::new(format!(
                "medium {} to be moved to permanent storage",
                medium_id
            ))
            .with_interval(Duration::from_millis(100))
            .with_exponential_backoff(2.0, Duration::from_secs(2))
            .with_timeout(Duration::from_secs(20)),
        )
        .await
    }
}
