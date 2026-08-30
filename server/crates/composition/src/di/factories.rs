use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use byte_unit::Byte;
use derive_new::new;
use event_sourcing::aggregate::repository::AggregateRepository;
use exif::{Exiftool, ExiftoolMetadataExtractor};
use filesystem::{FilesystemSettings, FilesystemStorageAdapter};
use kernel::{
    app_error::{ApplicationError, ApplicationResult},
    UserId,
};
use medium::{
    application::{
        ports::{FileStorage, MediumRepository, QuotaPort, Reservation},
        queries::MediumQueryPort,
        MediumApplicationHandlers,
    },
    domain::Medium,
};
use metadata::{
    application::{
        ports::{MetadataExtractor, MetadataRepository},
        queries::MetadataQueryPort,
        MetadataApplicationHandlers,
    },
    domain::Metadata,
};
use postgres::persistence::postgres::{
    es_snapshot_store::PostgresSnapshotStore,
    events::{aggregate_store::PostgresAggregateEventStore, type_registry::EventTypeRegistry},
    medium::PostgresMediumRepository,
    metadata::PostgresMetadataRepository,
    task::PostgresTaskRepository,
    user::{PostgresQuotaReservationStore, PostgresUserRepository},
};
use sqlx::PgPool;
use system::application::{AuthConfig, SystemApplicationHandlers};
use task::{
    application::{ports::TaskRepository, queries::TaskQueryPort, ProcessingApplicationHandlers},
    domain::Task,
};
use user::{
    application::{
        ports::{QuotaReservationStore, UserRepository},
        QuotaConfig, UserApplicationHandlers,
    },
    domain::{QuotaReservation, User},
    QuotaManager,
};

use crate::{
    config::GlobalConfig,
    di::stream_definitions::{medium_stream, metadata_stream, task_stream, user_stream},
    events::ProjectionEventBusAdapter,
};

// -- Helper structs --

pub struct Repositories {
    pub user: Arc<dyn UserRepository>,
    pub quota_reservations: Arc<dyn QuotaReservationStore>,
    pub medium: Arc<dyn MediumRepository>,
    pub medium_queries: Arc<dyn MediumQueryPort>,
    pub metadata: Arc<dyn MetadataRepository>,
    pub metadata_queries: Arc<dyn MetadataQueryPort>,
    pub task: Arc<dyn TaskRepository>,
    pub task_queries: Arc<dyn TaskQueryPort>,
}

pub struct StorageServices {
    pub file_storage: Arc<dyn FileStorage>,
    pub metadata_extractor: Arc<dyn MetadataExtractor>,
    pub storage_path_service: Arc<medium::domain::StoragePathService>,
}

/// Adapts the user module's `QuotaManager` to medium's `QuotaPort`
/// (ADR 0005). Composition is the only place that knows both concrete
/// types, so this keeps the module graph a star around the composition
/// root.
#[derive(new)]
pub struct UserQuotaPort {
    quota_manager: Arc<QuotaManager>,
}

#[async_trait]
impl QuotaPort for UserQuotaPort {
    async fn reserve(&self, user_id: UserId, bytes: Byte) -> ApplicationResult<Reservation> {
        let reservation = self.quota_manager.reserve(user_id, bytes).await?;
        Ok(Reservation::new(Box::new(reservation)))
    }

    async fn commit(&self, reservation: Reservation) -> ApplicationResult<()> {
        let reservation = unwrap_reservation(reservation)?;
        self.quota_manager.commit(reservation).await
    }

    async fn release(&self, reservation: Reservation) -> ApplicationResult<()> {
        let reservation = unwrap_reservation(reservation)?;
        self.quota_manager.release(reservation).await
    }
}

fn unwrap_reservation(reservation: Reservation) -> ApplicationResult<QuotaReservation> {
    reservation
        .downcast::<QuotaReservation>()
        .map_err(|_| ApplicationError::Internal {
            message: "quota reservation handle does not belong to the wired QuotaPort".to_string(),
        })
}

pub struct ApplicationHandlers {
    pub user: Arc<UserApplicationHandlers>,
    pub medium: Arc<MediumApplicationHandlers>,
    pub metadata: Arc<MetadataApplicationHandlers>,
    pub system: Arc<SystemApplicationHandlers>,
    pub processing: Arc<ProcessingApplicationHandlers>,
}

// -- Factory functions --

pub fn build_repositories(db_pool: &PgPool) -> Repositories {
    // The same concrete adapters implement both the write-side repository
    // ports and the read-side query ports (ADR 0002).
    let medium = Arc::new(PostgresMediumRepository::new(db_pool.clone()));
    let metadata = Arc::new(PostgresMetadataRepository::new(db_pool.clone()));
    let task = Arc::new(PostgresTaskRepository::new(db_pool.clone()));

    Repositories {
        user: Arc::new(PostgresUserRepository::new(db_pool.clone())),
        quota_reservations: Arc::new(PostgresQuotaReservationStore::new(db_pool.clone())),
        medium: medium.clone(),
        medium_queries: medium,
        metadata: metadata.clone(),
        metadata_queries: metadata,
        task: task.clone(),
        task_queries: task,
    }
}

pub async fn build_storage(config: Arc<GlobalConfig>) -> Result<StorageServices, snafu::Whatever> {
    let filesystem_settings = FilesystemSettings {
        base_path: config.storage.base_path.clone(),
        tmp_path: config.storage.tmp_path.clone(),
        cache_path: config.storage.cache_path.clone(),
    };
    let filesystem = Arc::new(FilesystemStorageAdapter::new(filesystem_settings));
    let exiftool = Arc::new(Exiftool::new().await?);
    let metadata_extractor = Arc::new(ExiftoolMetadataExtractor::new(exiftool, filesystem.clone()));
    let storage_path_service = Arc::new(medium::domain::StoragePathService::new(
        config.storage.pattern.clone(),
    ));

    Ok(StorageServices {
        file_storage: filesystem,
        metadata_extractor,
        storage_path_service,
    })
}

pub fn build_handlers(
    config: &Arc<GlobalConfig>,
    repositories: &Repositories,
    storage: &StorageServices,
    quota: Arc<dyn QuotaPort>,
    event_bus: Arc<ProjectionEventBusAdapter>,
) -> ApplicationHandlers {
    let quota_config = Arc::new(QuotaConfig {
        default_user_quota: Byte::from_u64(config.storage.default_user_quota),
        max_user_quota: Byte::from_u64(config.storage.max_user_quota),
    });

    let user_handlers = Arc::new(UserApplicationHandlers::new(
        repositories.user.clone(),
        event_bus.clone(),
        quota_config,
    ));

    let medium_handlers = Arc::new(MediumApplicationHandlers::new(
        repositories.medium.clone(),
        repositories.medium_queries.clone(),
        storage.file_storage.clone(),
        quota,
        event_bus.clone(),
        event_bus.clone(),
        storage.storage_path_service.clone(),
        event_bus.clone(),
    ));

    let metadata_handlers = Arc::new(MetadataApplicationHandlers::new(
        storage.metadata_extractor.clone(),
        repositories.metadata.clone(),
        repositories.metadata_queries.clone(),
        event_bus.clone(),
    ));

    let auth_config = Arc::new(AuthConfig {
        client_id: config.server.client_id.clone(),
        token_url: config.server.token_url.clone(),
        authorize_url: config.server.authorize_url.clone(),
    });

    let system_handlers = Arc::new(SystemApplicationHandlers::new(auth_config));

    let processing_handlers = Arc::new(ProcessingApplicationHandlers::new(
        repositories.task.clone(),
        repositories.task_queries.clone(),
    ));

    ApplicationHandlers {
        user: user_handlers,
        medium: medium_handlers,
        metadata: metadata_handlers,
        system: system_handlers,
        processing: processing_handlers,
    }
}

pub fn build_aggregate_repository(
    db_pool: &PgPool,
    registry: Arc<RwLock<EventTypeRegistry>>,
) -> Arc<AggregateRepository<i64>> {
    let store = Arc::new(PostgresAggregateEventStore::new(db_pool.clone(), registry));
    let snapshot_store = Arc::new(PostgresSnapshotStore::new(db_pool.clone()));

    let mut repo = AggregateRepository::new(store);
    repo.register_with_snapshots::<Medium>(medium_stream(), snapshot_store.clone());
    repo.register_with_snapshots::<User>(user_stream(), snapshot_store);
    repo.register::<Task>(task_stream());
    repo.register::<Metadata>(metadata_stream());

    Arc::new(repo)
}
