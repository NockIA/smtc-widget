use std::sync::Mutex;

type OnChange = Box<dyn Fn() + Send + Sync>;

/// Source audio choisie par l'utilisateur, partagée par toutes les commandes média.
pub struct AudioSourceState {
    /// `None` = session par défaut de Windows.
    selected: Mutex<Option<String>>,
    /// Appelé après un changement de source (le lecteur se réabonne à la nouvelle session).
    on_change: OnChange,
}

impl AudioSourceState {
    pub fn new(on_change: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            selected: Mutex::new(None),
            on_change: Box::new(on_change),
        }
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.lock().unwrap().clone()
    }

    pub fn select_source(&self, source_id: String) {
        *self.selected.lock().unwrap() = Some(source_id);
        (self.on_change)();
    }
}
