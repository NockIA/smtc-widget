use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

use super::constants::POSITION_EPSILON;

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

/// Préférences de fenêtre sauvegardées, une position par écran.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPreferences {
    /// Dernier écran sur lequel le widget se trouvait.
    pub last_monitor: Option<String>,
    /// Position personnalisée par écran (clé = `monitor_key`).
    pub monitors: HashMap<String, MonitorPlacement>,
}

impl WindowPreferences {
    /// Charge le fichier ; préférences vides s'il est absent ou illisible.
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)
    }
}
