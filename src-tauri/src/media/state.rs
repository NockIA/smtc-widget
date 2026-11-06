use std::sync::Mutex;
use std::sync::mpsc::Sender;

/// Commandes envoyées au thread d'écoute.
pub enum MediaCommand {
    /// La liste ou la session active a changé : se réabonner et réémettre.
    Refresh,
    /// Passe à la source audio suivante.
    NextSession,
}

/// État partagé, géré par Tauri.
pub struct MediaState {
    /// Source choisie par l'utilisateur ; `None` = session par défaut de Windows.
    selected: Mutex<Option<String>>,
    tx: Sender<MediaCommand>,
}

impl MediaState {
    pub fn new(tx: Sender<MediaCommand>) -> Self {
        Self {
            selected: Mutex::new(None),
            tx,
        }
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.lock().unwrap().clone()
    }

    pub fn set_selected(&self, source_id: String) {
        *self.selected.lock().unwrap() = Some(source_id);
    }

    /// Demande au thread d'écoute de passer à la source suivante.
    pub fn request_next_session(&self) {
        let _ = self.tx.send(MediaCommand::NextSession);
    }
}
