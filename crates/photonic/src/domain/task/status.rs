#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskTransition {
    Create,
    Start,
    Complete,
    Fail(String),
}

impl TaskStatus {
    pub fn transition(&self, transition: TaskTransition) -> Option<TaskStatus> {
        match (self, transition) {
            (TaskStatus::Pending, TaskTransition::Start) => Some(TaskStatus::InProgress),
            (TaskStatus::Pending, TaskTransition::Complete) => Some(TaskStatus::Completed),
            (TaskStatus::Pending, TaskTransition::Fail(reason)) => Some(TaskStatus::Failed(reason)),
            (TaskStatus::InProgress, TaskTransition::Complete) => Some(TaskStatus::Completed),
            (TaskStatus::InProgress, TaskTransition::Fail(reason)) => {
                Some(TaskStatus::Failed(reason))
            }
            // Invalid states
            _ => None,
        }
    }
}
