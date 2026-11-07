use serde::Deserialize;

use super::session::{resolve_session, smtc_manager};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaAction {
    TogglePlayPause,
    Previous,
    Next,
}

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
