use std::sync::{Arc, RwLock};

use application::{
    config::{AuthConfig, QuotaConfig},
    medium::{
        ports::{FileStorage, MediumRepository},
        MediumApplicationHandlers,
    },
    metadata::{
        ports::{MetadataExtractor, MetadataRepository},
        MetadataApplicationHandlers,
    },
    system::SystemApplicationHandlers,
    task::{ports::TaskRepository, ProcessingApplicationHandlers},
    user::{ports::UserRepository, QuotaManager, UserApplicationHandlers},
};
use byte_unit::Byte;
use domain::{medium::Medium, metadata::Metadata, task::Task, user::User};
use event_sourcing::aggregate::repository::AggregateRepository;
use sqlx::PgPool;

use crate::{
    config::GlobalConfig,
    di::stream_definitions::{medium_stream, metadata_stream, task_stream, user_stream},
    events::ProjectionEventBusAdapter,
    external::exif::{Exiftool, ExiftoolMetadataExtractor},
    persistence::postgres::{
        es_snapshot_store::PostgresSnapshotStore,
        events::{aggregate_store::PostgresAggregateEventStore, type_registry::EventTypeRegistry},
        medium::PostgresMediumRepository,
        metadata::PostgresMetadataRepository,
        task::PostgresTaskRepository,
        user::PostgresUserRepository,
    },
    storage::filesystem::repo::FilesystemStorageAdapter,
};

// -- Helper structs --

pub struct Repositories {
    pub user: Arc<dyn UserRepository>,
    pub medium: Arc<dyn MediumRepository>,
    pub metadata: Arc<dyn MetadataRepository>,
    pub task: Arc<dyn TaskRepository>,
}

pub struct StorageServices {
    pub file_storage: Arc<dyn FileStorage>,
    pub metadata_extractor: Arc<dyn MetadataExtractor>,
    pub storage_path_service: Arc<domain::medium::StoragePathService>,
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
    Repositories {
        user: Arc::new(PostgresUserRepository::new(db_pool.clone())),
        medium: Arc::new(PostgresMediumRepository::new(db_pool.clone())),
        metadata: Arc::new(PostgresMetadataRepository::new(db_pool.clone())),
        task: Arc::new(PostgresTaskRepository::new(db_pool.clone())),
    }
}

pub async fn build_storage(config: Arc<GlobalConfig>) -> Result<StorageServices, snafu::Whatever> {
    let filesystem = Arc::new(FilesystemStorageAdapter::new(config.clone()));
    let exiftool = Arc::new(Exiftool::new().await?);
    let metadata_extractor = Arc::new(ExiftoolMetadataExtractor::new(exiftool, filesystem.clone()));
    let storage_path_service = Arc::new(domain::medium::StoragePathService::new(
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
    quota_manager: Arc<QuotaManager>,
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
        storage.file_storage.clone(),
        quota_manager,
        event_bus.clone(),
        event_bus.clone(),
        storage.storage_path_service.clone(),
        event_bus.clone(),
    ));

    let metadata_handlers = Arc::new(MetadataApplicationHandlers::new(
        storage.metadata_extractor.clone(),
        repositories.metadata.clone(),
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
