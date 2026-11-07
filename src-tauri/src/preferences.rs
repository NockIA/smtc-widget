use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::theme::ThemePreference;

const PREFERENCES_FILE: &str = "preferences.json";

/// Écart (en pixels logiques) en dessous duquel deux positions sont considérées identiques.
const POSITION_EPSILON: f64 = 1.0;

/// Position du widget sur un écran, en marges (pixels logiques) par rapport
/// au coin bas-droit de cet écran. Indépendant de la résolution et du DPI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorPlacement {
    pub right_margin: f64,
    pub bottom_margin: f64,
}

impl MonitorPlacement {
    pub fn is_close_to(&self, other: &Self) -> bool {
        (self.right_margin - other.right_margin).abs() < POSITION_EPSILON
            && (self.bottom_margin - other.bottom_margin).abs() < POSITION_EPSILON
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub theme: ThemePreference,
    pub last_monitor: Option<String>,
    /// Position personnalisée par écran (clé = `monitor_key`).
    pub monitors: HashMap<String, MonitorPlacement>,
}

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
