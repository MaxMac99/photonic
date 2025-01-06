use crate::{
    error::Result,
    exif, medium,
    medium::MediumItemCreatedEvent,
    state::{AppState, ArcConnection},
    storage,
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

#[tracing::instrument(name = "run_medium_item_created_event_flow", skip(state))]
async fn run_flow(state: AppState, message: MediumItemCreatedEvent) -> Result<()> {
    let exif_event = exif::service::load_exif(state.clone(), message.clone()).await?;

    let mut conn1 = state.get_connection().await?;
    let conn1 = ArcConnection::new(&mut *conn1);
    let mut conn2 = state.get_connection().await?;
    let conn2 = ArcConnection::new(&mut *conn2);
    try_join!(
        storage::service::copy_medium_item_to_permanent(
            state.clone(),
            conn1.clone(),
            message.clone(),
            Some(exif_event.clone()),
        ),
        medium::service::update_medium_item_from_exif(conn2.clone(), exif_event.clone()),
    )?;
    storage::service::remove_medium_item_variant(
        state,
        conn1.clone(),
        message.id,
        storage::StorageVariant::Temp,
    )
    .await?;
    Ok(())
}
