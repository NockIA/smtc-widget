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

/// `None` s'il y a moins de deux sources.
pub fn next_source_id(manager: &SmtcManager, selected: Option<&str>) -> Option<String> {
    let ids: Vec<String> = all_sessions(manager).iter().map(source_id).collect();
    if ids.len() < 2 {
        return None;
    }

    let current = resolve_session(manager, selected)
        .map(|s| source_id(&s))
        .and_then(|id| ids.iter().position(|i| *i == id))
        .unwrap_or(0);

    Some(ids[(current + 1) % ids.len()].clone())
}
