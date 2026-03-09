use std::sync::Arc;

use snafu::Whatever;
use sqlx::PgPool;

use crate::application::medium::listeners::{
    MediumMetadataEnrichmentListener, MoveToPermanentStorageListener,
};
use crate::{
    application::{
        medium::{
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
    },
    domain::{
        medium::events::{
            MediumCreatedEvent, MediumUpdatedEvent, TempCleanupCompletedEvent,
            TempCleanupFailedEvent, TempCleanupStartedEvent,
        },
        metadata::events::{
            MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
        },
    },
    infrastructure::{
        config::GlobalConfig,
        events::EventBus,
        external::exif::{Exiftool, ExiftoolMetadataExtractor},
        persistence::postgres::{
            medium::PostgresMediumRepository, metadata::PostgresMetadataRepository,
            task::PostgresTaskRepository, user::PostgresUserRepository,
        },
        storage::{
            cleanup::spawn_cleanup_task,
            filesystem::{path_service::StoragePathService, repo::FilesystemStorageAdapter},
        },
    },
};

/// Dependency injection container
/// Manages the lifecycle and wiring of all application dependencies
pub struct Container {
    // Configuration
    config: Arc<GlobalConfig>,

    // Infrastructure - Event System
    event_bus: Arc<EventBus>,

    // Application - Handlers
    application_handlers: ApplicationHandlers,

    // Event Listeners (owned for lifecycle management)
    listener_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl Container {
    /// Create and initialize all dependencies
    pub async fn new(config: Arc<GlobalConfig>, db_pool: PgPool) -> Result<Arc<Self>, Whatever> {
        // Phase 1: Infrastructure adapters
        let repositories = Self::build_repositories(&db_pool);
        let storage = Self::build_storage(config.clone()).await?;

        // Phase 2: Event system
        let event_bus = Arc::new(EventBus::new());

        // Phase 3: Application services
        let quota_manager = Arc::new(QuotaManager::new(
            repositories.user.clone(),
            event_bus.clone(),
        ));

        // Phase 4: Application handlers
        let handlers = Self::build_handlers(
            &config,
            &repositories,
            &storage,
            quota_manager.clone(),
            event_bus.clone(),
        );

        // Phase 5: Start event listeners
        let mut listener_handles = Self::start_listeners(&handlers, &event_bus).await?;

        // Phase 6: Background tasks
        listener_handles.push(spawn_cleanup_task(
            handlers.medium.cleanup_expired_temp_storage.clone(),
            config.storage.temp_ttl_seconds,
            config.storage.cleanup_interval_seconds,
        ));

        Ok(Arc::new(Self {
            config,
            event_bus,
            application_handlers: handlers,
            listener_handles,
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

    /// Gracefully shutdown all event listeners
    pub async fn shutdown(self: Arc<Self>) {
        tracing::info!(
            "Shutting down {} event listeners...",
            self.listener_handles.len()
        );

        // We need to get ownership of the handles, but self is Arc
        // For now, abort the tasks - a proper implementation would use cancellation tokens
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
        let user_handlers = Arc::new(UserApplicationHandlers::new(
            repositories.user.clone(),
            event_bus.clone(),
            config.clone(),
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

        let system_handlers = Arc::new(SystemApplicationHandlers::new(config.clone()));

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
        let move_to_permanent_listener = MoveToPermanentStorageListener::new(
            handlers.medium.move_to_permanent_storage.clone(),
        );
        handles.extend(
            event_bus
                .start_processor::<MediumUpdatedEvent, _>(move_to_permanent_listener)
                .await?,
        );

        // TempCleanup task listeners - track cleanup sweep as a task
        let cleanup_started_listener =
            TaskStartedListeners::new(processing.start_task.clone());
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

        let cleanup_failed_listener =
            TaskFailedListeners::new(processing.fail_task.clone());
        handles.extend(
            event_bus
                .start_processor::<TempCleanupFailedEvent, _>(cleanup_failed_listener)
                .await?,
        );

        Ok(handles)
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

struct ApplicationHandlers {
    user: Arc<UserApplicationHandlers>,
    medium: Arc<MediumApplicationHandlers>,
    metadata: Arc<MetadataApplicationHandlers>,
    system: Arc<SystemApplicationHandlers>,
    processing: Option<Arc<ProcessingApplicationHandlers>>,
}
