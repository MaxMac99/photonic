pub mod commands;
pub mod ports;
pub mod queries;

use std::sync::Arc;

pub use ports::TaskRepository;
pub use queries::TaskQueryPort;

pub struct ProcessingApplicationHandlers {
    pub create_task: Arc<commands::CreateTaskHandler>,
    pub start_task: Arc<commands::StartTaskHandler>,
    pub complete_task: Arc<commands::CompleteTaskHandler>,
    pub fail_task: Arc<commands::FailTaskHandler>,
    pub find_tasks: Arc<queries::FindTasksHandler>,
}

impl ProcessingApplicationHandlers {
    pub fn new(repository: Arc<dyn TaskRepository>, task_queries: Arc<dyn TaskQueryPort>) -> Self {
        Self {
            create_task: Arc::new(commands::CreateTaskHandler::new(repository.clone())),
            start_task: Arc::new(commands::StartTaskHandler::new(repository.clone())),
            complete_task: Arc::new(commands::CompleteTaskHandler::new(repository.clone())),
            fail_task: Arc::new(commands::FailTaskHandler::new(repository)),
            find_tasks: Arc::new(queries::FindTasksHandler::new(task_queries)),
        }
    }
}
