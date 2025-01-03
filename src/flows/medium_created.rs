use crate::{
    error::Result, exif, medium::MediumItemCreatedEvent, state::AppState, storage,
    util::db::run_with_transaction,
};
use futures_util::{FutureExt, StreamExt};
use tokio::{task::JoinHandle, try_join};
use tracing::log::error;

pub fn setup_medium_created_flow(state: AppState) -> JoinHandle<()> {
    tokio::spawn(async move {
        state
            .event_bus
            .subscribe::<MediumItemCreatedEvent>()
            .await
            .for_each_concurrent(4, |event| async {
                if let Err(error) = run_flow(state.clone(), event).await {
                    error!("Error handling file created event: {}", error);
                }
            })
            .await;
        state
            .died
            .send(true)
            .await
            .expect("Could not send died signal");
    })
}

async fn run_flow(state: AppState, message: MediumItemCreatedEvent) -> Result<()> {
    let exif_event = exif::service::load_exif(state.clone(), message.clone()).await?;

    try_join!(run_with_transaction(state.clone(), |state, transaction| {
        storage::service::move_medium_item_to_permanent(
            state,
            transaction,
            message,
            Some(exif_event),
        )
        .boxed()
    }))?;
    Ok(())
}
