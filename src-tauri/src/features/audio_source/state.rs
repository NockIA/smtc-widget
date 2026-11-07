use std::sync::Mutex;
use std::sync::mpsc::Sender;

/// Source audio choisie par l'utilisateur, partagée par toutes les commandes média.
pub struct AudioSourceState {
    /// `None` = session par défaut de Windows.
    selected: Mutex<Option<String>>,
    /// Demande au thread d'écoute de se réabonner.
    refresh: Sender<()>,
}

impl AudioSourceState {
    pub fn new(refresh: Sender<()>) -> Self {
        Self {
            selected: Mutex::new(None),
            refresh,
        }
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.lock().unwrap().clone()
    }

    pub fn select_source(&self, source_id: String) {
        *self.selected.lock().unwrap() = Some(source_id);
        let _ = self.refresh.send(());
    }
}
