use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;

use super::models::TrackInfo;
use super::timeline::Timeline;
use crate::shared::smtc::{active_session, source_id};

pub async fn get_current_track(selected: Option<&str>) -> windows::core::Result<Option<TrackInfo>> {
    let Some(session) = active_session(selected)? else {
        return Ok(None);
    };

    let props = session.TryGetMediaPropertiesAsync()?.await?;
    let is_playing = session.GetPlaybackInfo()?.PlaybackStatus()?
        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
    let timeline = Timeline::estimate(&session, is_playing)?;

    Ok(Some(TrackInfo {
        title: props.Title()?.to_string(),
        artist: props.Artist()?.to_string(),
        album: props.AlbumTitle()?.to_string(),
        position_ms: timeline.position_ms,
        duration_ms: timeline.duration_ms,
        is_playing,
        source: source_id(&session),
    }))
}
