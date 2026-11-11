use tauri::WebviewWindow;

use crate::features::preferences::ThemePreference;

/// Applique le thème à la fenêtre, ce qui définit aussi `prefers-color-scheme` dans le webview.
/// « Système » et « Transparent » ne forcent rien : la fenêtre suit le thème de Windows.
pub fn apply_theme(window: &WebviewWindow, theme: ThemePreference) -> tauri::Result<()> {
    let tauri_theme = match theme {
        ThemePreference::System | ThemePreference::Transparent => None,
        ThemePreference::Light => Some(tauri::Theme::Light),
        ThemePreference::Dark => Some(tauri::Theme::Dark),
    };
    window.set_theme(tauri_theme)
}
