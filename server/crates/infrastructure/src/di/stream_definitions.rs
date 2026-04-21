use domain::{
    medium::{
        events::{MediumCreatedEvent, MediumItemCreatedEvent, MediumUpdatedEvent},
        Medium,
    },
    metadata::{
        events::{
            MetadataExtractedEvent, MetadataExtractionFailedEvent, MetadataExtractionStartedEvent,
        },
        Metadata,
    },
    task::{
        events::{TaskCompletedEvent, TaskCreatedEvent, TaskFailedEvent, TaskStartedEvent},
        Task,
    },
    user::{
        events::{
            QuotaCommittedEvent, QuotaReleasedEvent, QuotaReservedEvent, UserCreatedEvent,
            UserUpdatedEvent,
        },
        User,
    },
};
use event_sourcing::stream::definition::StreamDefinition;

pub fn medium_stream() -> StreamDefinition<Medium> {
    StreamDefinition::<Medium>::builder()
        .with::<MediumCreatedEvent>(|e| Some(e.medium_id.to_string()))
        .with::<MediumItemCreatedEvent>(|e| Some(e.medium_id.to_string()))
        .with::<MediumUpdatedEvent>(|e| Some(e.medium_id.to_string()))
        .build()
}

pub fn user_stream() -> StreamDefinition<User> {
    StreamDefinition::<User>::builder()
        .with::<UserCreatedEvent>(|e| Some(e.user_id.to_string()))
        .with::<UserUpdatedEvent>(|e| Some(e.user_id.to_string()))
        .with::<QuotaReservedEvent>(|e| Some(e.user_id.to_string()))
        .with::<QuotaCommittedEvent>(|e| Some(e.user_id.to_string()))
        .with::<QuotaReleasedEvent>(|e| Some(e.user_id.to_string()))
        .build()
}

pub fn task_stream() -> StreamDefinition<Task> {
    StreamDefinition::<Task>::builder()
        .with::<TaskCreatedEvent>(|e| Some(e.task_id.to_string()))
        .with::<TaskStartedEvent>(|e| Some(e.task_id.to_string()))
        .with::<TaskCompletedEvent>(|e| Some(e.task_id.to_string()))
        .with::<TaskFailedEvent>(|e| Some(e.task_id.to_string()))
        .build()
}

pub fn metadata_stream() -> StreamDefinition<Metadata> {
    StreamDefinition::<Metadata>::builder()
        .with::<MetadataExtractionStartedEvent>(|e| Some(e.medium_id.to_string()))
        .with::<MetadataExtractedEvent>(|e| Some(e.medium_id.to_string()))
        .with::<MetadataExtractionFailedEvent>(|e| Some(e.medium_id.to_string()))
        .build()
}
