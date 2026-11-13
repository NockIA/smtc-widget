use std::sync::OnceLock;

use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
};

pub type Session = GlobalSystemMediaTransportControlsSession;
pub type SmtcManager = GlobalSystemMediaTransportControlsSessionManager;

static MANAGER: OnceLock<SmtcManager> = OnceLock::new();

/// Manager SMTC partagé : créé au premier appel, puis réutilisé.
pub fn smtc_manager() -> windows::core::Result<SmtcManager> {
    if let Some(manager) = MANAGER.get() {
        return Ok(manager.clone());
    }
    let manager = SmtcManager::RequestAsync()?.join()?;
    Ok(MANAGER.get_or_init(|| manager).clone())
}

pub fn source_id(session: &Session) -> String {
    session
        .SourceAppUserModelId()
        .inspect_err(|e| log::warn!("lecture de l'identifiant de source a échoué : {e}"))
        .map_or_else(|_| String::new(), |id| id.to_string())
}

pub fn all_sessions(manager: &SmtcManager) -> Vec<Session> {
    manager
        .GetSessions()
        .inspect_err(|e| log::warn!("liste des sessions SMTC illisible : {e}"))
        .map_or_else(|_| Vec::new(), |sessions| sessions.into_iter().collect())
}

/// Session active (voir `resolve_session`), en créant le manager au besoin.
pub fn active_session(selected: Option<&str>) -> windows::core::Result<Option<Session>> {
    Ok(resolve_session(&smtc_manager()?, selected))
}

/// Session choisie par l'utilisateur si elle existe encore, sinon celle de Windows.
pub fn resolve_session(manager: &SmtcManager, selected: Option<&str>) -> Option<Session> {
    selected
        .and_then(|id| all_sessions(manager).into_iter().find(|s| source_id(s) == id))
        .or_else(|| manager.GetCurrentSession().ok())
}
