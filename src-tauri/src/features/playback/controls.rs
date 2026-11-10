use super::models::MediaAction;
use crate::shared::smtc::active_session;

/// `false` si aucune session n'existe ou si la source ne supporte pas l'action.
pub async fn send_action(selected: Option<&str>, action: MediaAction) -> windows::core::Result<bool> {
    let Some(session) = active_session(selected)? else {
        return Ok(false);
    };

    match action {
        MediaAction::TogglePlayPause => session.TryTogglePlayPauseAsync()?.await,
        MediaAction::Previous => session.TrySkipPreviousAsync()?.await,
        MediaAction::Next => session.TrySkipNextAsync()?.await,
    }
}
