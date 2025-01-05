use crate::{
    error::Result, exif, medium, medium::MediumItemCreatedEvent, state::AppState, storage,
};
use futures_util::StreamExt;
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

#[tracing::instrument(skip(state))]
async fn run_flow(state: AppState, message: MediumItemCreatedEvent) -> Result<()> {
    let exif_event = exif::service::load_exif(state.clone(), message.clone()).await?;

    let mut conn1 = state.get_connection().await?;
    let mut conn2 = state.get_connection().await?;
    try_join!(
        storage::service::move_medium_item_to_permanent(
            state.clone(),
            &mut *conn1,
            message,
            Some(exif_event.clone()),
        ),
        medium::service::update_medium_item_from_exif(&mut *conn2, exif_event.clone()),
    )?;
    Ok(())
}
