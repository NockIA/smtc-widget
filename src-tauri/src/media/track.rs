use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;

use super::models::TrackInfo;
use super::session::{all_sessions, request_manager, resolve_session, source_id};
use super::timeline::Timeline;

pub fn get_current_track(selected: Option<&str>) -> windows::core::Result<Option<TrackInfo>> {
    let manager = request_manager()?;

    let Some(session) = resolve_session(&manager, selected) else {
        return Ok(None);
    };

    let props = session.TryGetMediaPropertiesAsync()?.join()?;
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
        session_count: all_sessions(&manager).len(),
    }))
}
