use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{fs, io};

use super::constants::PREFERENCES_FILE;
use super::models::Preferences;

/// Préférences partagées entre threads : un seul verrou, un seul fichier.
pub struct PreferencesStore {
    path: PathBuf,
    data: Mutex<Preferences>,
}

impl PreferencesStore {
    /// Charge le fichier du dossier de config ; préférences par défaut s'il est absent ou illisible.
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join(PREFERENCES_FILE);
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();
        Self {
            path,
            data: Mutex::new(data),
        }
    }

    /// Copie des préférences, pour travailler sans garder le verrou.
    pub fn snapshot(&self) -> Preferences {
        self.data.lock().unwrap().clone()
    }

    /// Modifie les préférences et les sauvegarde.
    pub fn update(&self, change: impl FnOnce(&mut Preferences)) -> io::Result<()> {
        let mut data = self.data.lock().unwrap();
        change(&mut data);

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, serde_json::to_string_pretty(&*data)?)
    }
}
