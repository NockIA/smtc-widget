use crate::features::playback::models::MediaAction;
use crate::shared::smtc::active_session;

/// `false` si aucune session n'existe ou si la source ne supporte pas l'action.
pub fn send_action(selected: Option<&str>, action: MediaAction) -> windows::core::Result<bool> {
    let Some(session) = active_session(selected)? else {
        return Ok(false);
    };

    match action {
        MediaAction::TogglePlayPause => session.TryTogglePlayPauseAsync()?.join(),
        MediaAction::Previous => session.TrySkipPreviousAsync()?.join(),
        MediaAction::Next => session.TrySkipNextAsync()?.join(),
    }
}
