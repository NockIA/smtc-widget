use super::models::TrackInfo;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

pub fn get_current_track() -> windows::core::Result<Option<TrackInfo>> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()?;

    let session = match manager.GetCurrentSession() {
        Ok(session) => session,
        Err(_) => return Ok(None),
    };

    let props = session.TryGetMediaPropertiesAsync()?.join()?;

    let title = props.Title()?.to_string();
    let artist = props.Artist()?.to_string();
    let album = props.AlbumTitle()?.to_string();

    Ok(Some(TrackInfo {
        title,
        artist,
        album,
    }))
}
