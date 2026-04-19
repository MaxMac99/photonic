use std::sync::Arc;

use application::{
    medium::MediumApplicationHandlers,
    metadata::MetadataApplicationHandlers,
    system::SystemApplicationHandlers,
    task::ProcessingApplicationHandlers,
    user::{QuotaManager, UserApplicationHandlers},
};
use event_sourcing::aggregate::repository::AggregateRepository;
use snafu::{ResultExt, Whatever};
use sqlx::PgPool;

use super::{
    event_system::build_projection_bus,
    factories::{
        build_aggregate_repository, build_handlers, build_repositories, build_storage,
        ApplicationHandlers,
    },
    listeners::register_listeners,
};
use crate::{
    config::GlobalConfig, events::ProjectionEventBusAdapter, storage::cleanup::spawn_cleanup_task,
};

/// Dependency injection container.
/// Manages the lifecycle and wiring of all application dependencies.
pub struct Container {
    config: Arc<GlobalConfig>,
    aggregate_repo: Arc<AggregateRepository<i64>>,
    application_handlers: ApplicationHandlers,
    background_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Container {
    pub async fn new(config: Arc<GlobalConfig>, db_pool: PgPool) -> Result<Arc<Self>, Whatever> {
        // Phase 1: Infrastructure adapters
        let repositories = build_repositories(&db_pool);
        let storage = build_storage(config.clone()).await?;

        // Phase 2: Event system (projection bus + auto-populated registry)
        let (bus, registry) = build_projection_bus(&db_pool)?;

        // Phase 3: Application services
        let bus_adapter = Arc::new(ProjectionEventBusAdapter::new(bus.clone()));
        let quota_manager = Arc::new(QuotaManager::new(
            repositories.user.clone(),
            bus_adapter.clone(),
        ));
        let handlers = build_handlers(&config, &repositories, &storage, quota_manager, bus_adapter);

        // Phase 4: Register listeners (as checkpointed projection handlers)
        {
            let mut reg = registry.write().unwrap();
            register_listeners(&handlers, &bus, &mut reg)
                .whatever_context("Failed to register listeners")?;
        }

        // Phase 5: Aggregate repository (uses shared registry)
        let aggregate_repo = build_aggregate_repository(&db_pool, registry);

        // Phase 6: Replay all projections + listeners from checkpoints
        bus.start()
            .await
            .whatever_context("Failed to start ProjectionEventBus")?;

        // Phase 7: Background tasks
        let mut background_tasks = Vec::new();
        background_tasks.push(spawn_cleanup_task(
            handlers.medium.cleanup_expired_temp_storage.clone(),
            config.storage.temp_ttl_seconds,
            config.storage.cleanup_interval_seconds,
        ));

        Ok(Arc::new(Self {
            config,
            aggregate_repo,
            application_handlers: handlers,
            background_tasks,
        }))
    }

    // -- Accessors --

    pub fn config(&self) -> Arc<GlobalConfig> {
        self.config.clone()
    }

    pub fn aggregate_repo(&self) -> &Arc<AggregateRepository<i64>> {
        &self.aggregate_repo
    }

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

    pub fn processing_handlers(&self) -> Arc<ProcessingApplicationHandlers> {
        self.application_handlers.processing.clone()
    }

    pub async fn shutdown(self: Arc<Self>) {
        tracing::info!(
            "Shutting down {} background tasks...",
            self.background_tasks.len()
        );
        for handle in &self.background_tasks {
            handle.abort();
        }
        tracing::info!("All background tasks shut down");
    }
}
