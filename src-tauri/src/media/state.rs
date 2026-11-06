use std::sync::Mutex;
use std::sync::mpsc::Sender;

pub struct MediaState {
    /// Source choisie par l'utilisateur ; `None` = session par défaut de Windows.
    selected: Mutex<Option<String>>,
    tx: Sender<()>,
}

impl MediaState {
    pub fn new(tx: Sender<()>) -> Self {
        Self {
            selected: Mutex::new(None),
            tx,
        }
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.lock().unwrap().clone()
    }

    /// Choisit la source et demande au thread d'écoute de se réabonner.
    pub fn select_source(&self, source_id: String) {
        *self.selected.lock().unwrap() = Some(source_id);
        let _ = self.tx.send(());
    }
}
