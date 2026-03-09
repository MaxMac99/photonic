pub mod commands;
pub mod listeners;
pub mod ports;
mod queries;

use std::sync::Arc;

pub use ports::TaskRepository;

pub struct ProcessingApplicationHandlers {
    pub create_task: Arc<commands::CreateTaskHandler>,
    pub start_task: Arc<commands::StartTaskHandler>,
    pub complete_task: Arc<commands::CompleteTaskHandler>,
    pub fail_task: Arc<commands::FailTaskHandler>,
}

impl ProcessingApplicationHandlers {
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self {
            create_task: Arc::new(commands::CreateTaskHandler::new(repository.clone())),
            start_task: Arc::new(commands::StartTaskHandler::new(repository.clone())),
            complete_task: Arc::new(commands::CompleteTaskHandler::new(repository.clone())),
            fail_task: Arc::new(commands::FailTaskHandler::new(repository)),
        }
    }
}
