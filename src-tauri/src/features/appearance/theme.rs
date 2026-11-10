use tauri::WebviewWindow;

use crate::features::preferences::ThemePreference;

/// Le thème de la fenêtre pilote `prefers-color-scheme` dans le webview ; `None` suit Windows.
pub fn apply_theme(window: &WebviewWindow, theme: ThemePreference) -> tauri::Result<()> {
    let tauri_theme = match theme {
        ThemePreference::System | ThemePreference::Transparent => None,
        ThemePreference::Light => Some(tauri::Theme::Light),
        ThemePreference::Dark => Some(tauri::Theme::Dark),
    };
    window.set_theme(tauri_theme)
}
