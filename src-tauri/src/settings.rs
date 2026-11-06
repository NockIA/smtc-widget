use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{fs, io};

use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

pub const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Système",
            Self::Light => "Clair",
            Self::Dark => "Sombre",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.id() == id)
    }

    fn to_tauri(self) -> Option<tauri::Theme> {
        match self {
            Self::System => None,
            Self::Light => Some(tauri::Theme::Light),
            Self::Dark => Some(tauri::Theme::Dark),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub theme: ThemePreference,
}

impl Settings {
    fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)
    }
}

/// Réglages de l'utilisateur, gérés par Tauri et sauvegardés à chaque modification.
pub struct SettingsState {
    path: PathBuf,
    settings: Mutex<Settings>,
}

impl SettingsState {
    pub fn load(path: PathBuf) -> Self {
        let settings = Mutex::new(Settings::load(&path));
        Self { path, settings }
    }

    pub fn theme(&self) -> ThemePreference {
        self.settings.lock().unwrap().theme
    }

    pub fn set_theme(&self, theme: ThemePreference) -> io::Result<()> {
        let mut settings = self.settings.lock().unwrap();
        settings.theme = theme;
        settings.save(&self.path)
    }
}

/// Le thème de la fenêtre pilote `prefers-color-scheme` dans le webview ; `None` suit Windows.
pub fn apply_theme(window: &WebviewWindow, theme: ThemePreference) -> tauri::Result<()> {
    window.set_theme(theme.to_tauri())
}
