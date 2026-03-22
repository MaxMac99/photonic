use std::{sync::Arc, time::Duration};

use application::{
    aggregate_repository::AggregateRepository,
    config::{AuthConfig, QuotaConfig},
    medium::{
        events::{TempCleanupCompletedEvent, TempCleanupFailedEvent, TempCleanupStartedEvent},
        listeners::{MediumMetadataEnrichmentListener, MoveToPermanentStorageListener},
        ports::{FileStorage, MediumRepository},
        MediumApplicationHandlers,
    },
    metadata::{
        listeners::MetadataExtractionListeners,
        ports::{MetadataExtractor, MetadataRepository},
        MetadataApplicationHandlers,
    },
    system::SystemApplicationHandlers,
    task::{
        listeners::{
            TaskCompletedListeners, TaskCreationListeners, TaskFailedListeners,
            TaskStartedListeners,
        },
        ports::TaskRepository,
        ProcessingApplicationHandlers,
    },
    user::{ports::UserRepository, QuotaManager, UserApplicationHandlers},
};
use byte_unit::Byte;
use domain::{
    medium::{
        events::{MediumCreatedEvent, MediumUpdatedEvent},
        Medium, StoragePathService,
    },
    metadata::{
        events::{
            MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
        },
        Metadata,
    },
    task::Task,
    user::User,
};
use snafu::Whatever;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::{
    config::GlobalConfig,
    events::EventBus,
    external::exif::{Exiftool, ExiftoolMetadataExtractor},
    persistence::postgres::{
        event_store::PostgresEventStore, medium::PostgresMediumRepository,
        metadata::PostgresMetadataRepository, snapshot_store::PostgresSnapshotStore,
        task::PostgresTaskRepository, user::PostgresUserRepository,
    },
    projections::{
        MediumProjection, MetadataProjection, ProjectionEngine, TaskProjection, UserProjection,
    },
    storage::{cleanup::spawn_cleanup_task, filesystem::repo::FilesystemStorageAdapter},
};

/// Dependency injection container
/// Manages the lifecycle and wiring of all application dependencies
pub struct Container {
    // Configuration
    config: Arc<GlobalConfig>,

    // Infrastructure - Event System
    event_bus: Arc<EventBus>,

    // Event Sourcing - Aggregate Repositories
    medium_aggregate_repo: Arc<AggregateRepository<Medium>>,
    user_aggregate_repo: Arc<AggregateRepository<User>>,
    task_aggregate_repo: Arc<AggregateRepository<Task>>,
    metadata_aggregate_repo: Arc<AggregateRepository<Metadata>>,

    // Application - Handlers
    application_handlers: ApplicationHandlers,

    // Event Listeners (owned for lifecycle management)
    listener_handles: Vec<tokio::task::JoinHandle<()>>,

    // Projection engine shutdown
    projection_shutdown: CancellationToken,
}

impl Container {
    /// Create and initialize all dependencies
    pub async fn new(config: Arc<GlobalConfig>, db_pool: PgPool) -> Result<Arc<Self>, Whatever> {
        // Phase 1: Infrastructure adapters
        let repositories = Self::build_repositories(&db_pool);
        let storage = Self::build_storage(config.clone()).await?;

        // Phase 2: Event system
        let event_bus = Arc::new(EventBus::new());

        // Phase 3: Event sourcing infrastructure
        let aggregate_repos = Self::build_aggregate_repositories(&db_pool);
        let projection_shutdown =
            Self::start_projection_engine(&db_pool, &aggregate_repos).await;

        // Phase 4: Application services
        let quota_manager = Arc::new(QuotaManager::new(
            repositories.user.clone(),
            event_bus.clone(),
        ));

        // Phase 5: Application handlers
        let handlers = Self::build_handlers(
            &config,
            &repositories,
            &storage,
            quota_manager.clone(),
            event_bus.clone(),
        );

        // Phase 6: Start event listeners
        let mut listener_handles = Self::start_listeners(&handlers, &event_bus).await?;

        // Phase 7: Background tasks
        listener_handles.push(spawn_cleanup_task(
            handlers.medium.cleanup_expired_temp_storage.clone(),
            config.storage.temp_ttl_seconds,
            config.storage.cleanup_interval_seconds,
        ));

        Ok(Arc::new(Self {
            config,
            event_bus,
            medium_aggregate_repo: aggregate_repos.medium,
            user_aggregate_repo: aggregate_repos.user,
            task_aggregate_repo: aggregate_repos.task,
            metadata_aggregate_repo: aggregate_repos.metadata,
            application_handlers: handlers,
            listener_handles,
            projection_shutdown,
        }))
    }

    // Accessors for handlers (used by AppState)
    pub fn user_handlers(&self) -> Arc<UserApplicationHandlers> {
        self.application_handlers.user.clone()
    }

    pub fn medium_handlers(&self) -> Arc<MediumApplicationHandlers> {
        self.application_handlers.medium.clone()
    }

    pub fn metadata_handlers(&self) -> Arc<MetadataApplicationHandlers> {
        self.application_handlers.metadata.clone()
    }

    pub fn system_handlers(&self) -> Arc<SystemApplicationHandlers> {
        self.application_handlers.system.clone()
    }

    pub fn processing_handlers(&self) -> Option<Arc<ProcessingApplicationHandlers>> {
        self.application_handlers.processing.clone()
    }

    pub fn config(&self) -> Arc<GlobalConfig> {
        self.config.clone()
    }

    // Event sourcing accessors
    pub fn medium_aggregate_repo(&self) -> Arc<AggregateRepository<Medium>> {
        self.medium_aggregate_repo.clone()
    }

    pub fn user_aggregate_repo(&self) -> Arc<AggregateRepository<User>> {
        self.user_aggregate_repo.clone()
    }

    pub fn task_aggregate_repo(&self) -> Arc<AggregateRepository<Task>> {
        self.task_aggregate_repo.clone()
    }

    pub fn metadata_aggregate_repo(&self) -> Arc<AggregateRepository<Metadata>> {
        self.metadata_aggregate_repo.clone()
    }

    /// Gracefully shutdown all event listeners and projection engine
    pub async fn shutdown(self: Arc<Self>) {
        tracing::info!("Shutting down projection engine...");
        self.projection_shutdown.cancel();

        tracing::info!(
            "Shutting down {} event listeners...",
            self.listener_handles.len()
        );

        for handle in &self.listener_handles {
            handle.abort();
        }

        tracing::info!("All event listeners shut down");
    }

    // Private factory methods

    fn build_repositories(db_pool: &PgPool) -> Repositories {
        Repositories {
            user: Arc::new(PostgresUserRepository::new(db_pool.clone())),
            medium: Arc::new(PostgresMediumRepository::new(db_pool.clone())),
            metadata: Arc::new(PostgresMetadataRepository::new(db_pool.clone())),
            task: Arc::new(PostgresTaskRepository::new(db_pool.clone())),
        }
    }

    async fn build_storage(config: Arc<GlobalConfig>) -> Result<StorageServices, Whatever> {
        let filesystem = Arc::new(FilesystemStorageAdapter::new(config.clone()));
        let exiftool = Arc::new(Exiftool::new().await?);
        let metadata_extractor =
            Arc::new(ExiftoolMetadataExtractor::new(exiftool, filesystem.clone()));
        let storage_path_service =
            Arc::new(StoragePathService::new(config.storage.pattern.clone()));

        Ok(StorageServices {
            file_storage: filesystem,
            metadata_extractor,
            storage_path_service,
        })
    }

    fn build_handlers(
        config: &Arc<GlobalConfig>,
        repositories: &Repositories,
        storage: &StorageServices,
        quota_manager: Arc<QuotaManager>,
        event_bus: Arc<EventBus>,
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
            processing: Some(processing_handlers),
        }
    }

    async fn start_listeners(
        handlers: &ApplicationHandlers,
        event_bus: &Arc<EventBus>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, Whatever> {
        let mut handles = Vec::new();

        let processing = handlers
            .processing
            .as_ref()
            .expect("ProcessingApplicationHandlers must be initialized");

        // TaskCreationListener - creates pending task when medium is created
        let task_creation_listener = TaskCreationListeners::new(processing.create_task.clone());
        handles.extend(
            event_bus
                .start_processor::<MediumCreatedEvent, _>(task_creation_listener)
                .await?,
        );

        // MetadataExtractionListener - starts extraction when medium is created
        let metadata_listener =
            MetadataExtractionListeners::new(handlers.metadata.extract_metadata_handler.clone());
        handles.extend(
            event_bus
                .start_processor::<MediumCreatedEvent, _>(metadata_listener)
                .await?,
        );

        // TaskStartedListeners - transitions task to InProgress when extraction starts
        let started_listener = TaskStartedListeners::new(processing.start_task.clone());
        handles.extend(
            event_bus
                .start_processor::<MetadataExtractionStartedEvent, _>(started_listener)
                .await?,
        );

        // TaskCompletedListeners - transitions task to Completed when extraction succeeds
        let completed_listener = TaskCompletedListeners::new(processing.complete_task.clone());
        handles.extend(
            event_bus
                .start_processor::<MetadataExtractedEvent, _>(completed_listener)
                .await?,
        );

        // TaskFailedListeners - transitions task to Failed when extraction fails
        let failed_listener = TaskFailedListeners::new(processing.fail_task.clone());
        handles.extend(
            event_bus
                .start_processor::<MetadataExtractionFailedEvent, _>(failed_listener)
                .await?,
        );

        let medium_metadata_enrichment_listener = MediumMetadataEnrichmentListener::new(
            handlers.medium.enrich_medium_with_metadata.clone(),
        );
        handles.extend(
            event_bus
                .start_processor::<MetadataExtractedEvent, _>(medium_metadata_enrichment_listener)
                .await?,
        );

        // MoveToPermanentStorageListener - moves items to permanent storage when medium is updated
        let move_to_permanent_listener =
            MoveToPermanentStorageListener::new(handlers.medium.move_to_permanent_storage.clone());
        handles.extend(
            event_bus
                .start_processor::<MediumUpdatedEvent, _>(move_to_permanent_listener)
                .await?,
        );

        // TempCleanup task listeners - track cleanup sweep as a task
        let cleanup_started_listener = TaskStartedListeners::new(processing.start_task.clone());
        handles.extend(
            event_bus
                .start_processor::<TempCleanupStartedEvent, _>(cleanup_started_listener)
                .await?,
        );

        let cleanup_completed_listener =
            TaskCompletedListeners::new(processing.complete_task.clone());
        handles.extend(
            event_bus
                .start_processor::<TempCleanupCompletedEvent, _>(cleanup_completed_listener)
                .await?,
        );

        let cleanup_failed_listener = TaskFailedListeners::new(processing.fail_task.clone());
        handles.extend(
            event_bus
                .start_processor::<TempCleanupFailedEvent, _>(cleanup_failed_listener)
                .await?,
        );

        Ok(handles)
    }

    fn build_aggregate_repositories(db_pool: &PgPool) -> AggregateRepositories {
        let snapshot_interval = 100; // snapshot every 100 events

        let medium_event_store = Arc::new(PostgresEventStore::<Medium>::new(db_pool.clone()));
        let medium_snapshot_store =
            Arc::new(PostgresSnapshotStore::<Medium>::new(db_pool.clone()));
        let medium_repo = Arc::new(AggregateRepository::new(
            medium_event_store,
            Some(medium_snapshot_store),
            snapshot_interval,
        ));

        let user_event_store = Arc::new(PostgresEventStore::<User>::new(db_pool.clone()));
        let user_snapshot_store = Arc::new(PostgresSnapshotStore::<User>::new(db_pool.clone()));
        let user_repo = Arc::new(AggregateRepository::new(
            user_event_store,
            Some(user_snapshot_store),
            snapshot_interval,
        ));

        let task_event_store = Arc::new(PostgresEventStore::<Task>::new(db_pool.clone()));
        let task_repo = Arc::new(AggregateRepository::new(task_event_store, None, 0));

        let metadata_event_store =
            Arc::new(PostgresEventStore::<Metadata>::new(db_pool.clone()));
        let metadata_repo = Arc::new(AggregateRepository::new(metadata_event_store, None, 0));

        AggregateRepositories {
            medium: medium_repo,
            user: user_repo,
            task: task_repo,
            metadata: metadata_repo,
        }
    }

    async fn start_projection_engine(
        db_pool: &PgPool,
        _aggregate_repos: &AggregateRepositories,
    ) -> CancellationToken {
        use domain::{
            medium::events::MediumEvent, metadata::events::MetadataEvent,
            task::events::TaskEvent, user::events::UserEvent,
        };

        let mut engine = ProjectionEngine::new(
            db_pool.clone(),
            Duration::from_millis(100),
            100,
        );

        // Register read model projections
        engine.register::<MediumEvent, _>(MediumProjection::new(db_pool.clone()));
        engine.register::<UserEvent, _>(UserProjection::new(db_pool.clone()));
        engine.register::<MetadataEvent, _>(MetadataProjection::new(db_pool.clone()));
        engine.register::<TaskEvent, _>(TaskProjection::new(db_pool.clone()));

        // Start engine in background
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            engine.run(shutdown_clone).await;
        });

        shutdown
    }
}

// Helper structs for organizing dependencies

struct Repositories {
    user: Arc<dyn UserRepository>,
    medium: Arc<dyn MediumRepository>,
    metadata: Arc<dyn MetadataRepository>,
    task: Arc<dyn TaskRepository>,
}

struct StorageServices {
    file_storage: Arc<dyn FileStorage>,
    metadata_extractor: Arc<dyn MetadataExtractor>,
    storage_path_service: Arc<StoragePathService>,
}

struct AggregateRepositories {
    medium: Arc<AggregateRepository<Medium>>,
    user: Arc<AggregateRepository<User>>,
    task: Arc<AggregateRepository<Task>>,
    metadata: Arc<AggregateRepository<Metadata>>,
}

struct ApplicationHandlers {
    user: Arc<UserApplicationHandlers>,
    medium: Arc<MediumApplicationHandlers>,
    metadata: Arc<MetadataApplicationHandlers>,
    system: Arc<SystemApplicationHandlers>,
    processing: Option<Arc<ProcessingApplicationHandlers>>,
}
