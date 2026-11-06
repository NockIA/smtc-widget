use std::sync::Mutex;
use std::sync::mpsc::Sender;

pub enum MediaCommand {
    Refresh,
    NextSession,
}

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

    pub fn request_next_session(&self) {
        let _ = self.tx.send(MediaCommand::NextSession);
    }
}
