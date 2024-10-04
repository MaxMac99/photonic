use crate::{error::NoDateTakenSnafu, server::AppState, store::PathOptions};
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use meta::MetaInfo;
use snafu::OptionExt;
use std::{fmt::Debug, path::Path};
use tokio::{fs, join};
use uuid::Uuid;

pub(crate) async fn create_path_options<P>(
    app_state: AppState,
    path: P,
    username: String,
    album_id: Option<Uuid>,
    date_taken: Option<DateTime<FixedOffset>>,
    filename: String,
    extension: String,
) -> crate::error::Result<(u64, MetaInfo, PathOptions)>
where
    P: AsRef<Path> + Debug,
{
    let (size, meta_info, album) = join!(
        fs::metadata(&path),
        app_state.meta.read_file(&path, true),
        get_album_by_id(album_id)
    );
    let size = size?.len();
    let meta_info = meta_info?;

    let date_taken = date_taken.or(meta_info.date).context(NoDateTakenSnafu)?;
    let timezone = date_taken.timezone().local_minus_utc();
    let date_taken: DateTime<Utc> = date_taken.into();

    let album = album?;
    let name = album.as_ref().map(|album| album.name.clone());
    let year = album
        .as_ref()
        .and_then(|album| album.first_date)
        .map(|date| date.year() as u32);
    let path_opts = PathOptions {
        username,
        album: name,
        album_year: year,
        date: date_taken,
        camera_make: meta_info.camera_make.clone(),
        camera_model: meta_info.camera_model.clone(),
        timezone,
        filename,
        extension,
    };
    Ok((size, meta_info, path_opts))
}
