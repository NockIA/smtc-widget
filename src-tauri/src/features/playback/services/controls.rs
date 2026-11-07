use crate::features::playback::models::MediaAction;
use crate::shared::smtc::{resolve_session, smtc_manager};

/// `false` si aucune session n'existe ou si la source ne supporte pas l'action.
pub fn send_action(selected: Option<&str>, action: MediaAction) -> windows::core::Result<bool> {
    let manager = smtc_manager()?;

    let Some(session) = resolve_session(&manager, selected) else {
        return Ok(false);
    };

    match action {
        MediaAction::TogglePlayPause => session.TryTogglePlayPauseAsync()?.join(),
        MediaAction::Previous => session.TrySkipPreviousAsync()?.join(),
        MediaAction::Next => session.TrySkipNextAsync()?.join(),
    }
}
