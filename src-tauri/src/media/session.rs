use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
};

pub type Session = GlobalSystemMediaTransportControlsSession;
pub type SmtcManager = GlobalSystemMediaTransportControlsSessionManager;

pub fn request_manager() -> windows::core::Result<SmtcManager> {
    SmtcManager::RequestAsync()?.join()
}

pub fn source_id(session: &Session) -> String {
    session
        .SourceAppUserModelId()
        .map(|id| id.to_string())
        .unwrap_or_default()
}

pub fn all_sessions(manager: &SmtcManager) -> Vec<Session> {
    manager
        .GetSessions()
        .map(|sessions| sessions.into_iter().collect())
        .unwrap_or_default()
}

/// Session choisie par l'utilisateur si elle existe encore, sinon celle de Windows.
pub fn resolve_session(manager: &SmtcManager, selected: Option<&str>) -> Option<Session> {
    selected
        .and_then(|id| {
            all_sessions(manager)
                .into_iter()
                .find(|s| source_id(s) == id)
        })
        .or_else(|| manager.GetCurrentSession().ok())
}
