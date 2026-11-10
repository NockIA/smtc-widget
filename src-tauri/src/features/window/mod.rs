mod foreground;
mod layout;
mod models;
mod persistence;
mod placement;
mod taskbar_layout;
mod webview_memory;
mod win32;

use tauri::{Manager, WebviewWindow};

use crate::features::preferences::PreferencesStore;

pub use foreground::{is_hidden, keep_on_top, pause_keep_on_top};
pub use placement::list_monitors;

/// Place la fenêtre selon les préférences, l'affiche, puis suit ses déplacements et les changements d'écrans.
pub fn init_placement(window: WebviewWindow) {
    let prefs = window.state::<PreferencesStore>().snapshot();

    if let Err(e) = placement::place_on_preferred_monitor(&window, &prefs) {
        log::error!("placement de la fenêtre a échoué : {e}");
    }
    // Cachée dans la config jusqu'ici, pour ne pas apparaître à une position provisoire.
    if let Err(e) = window.show() {
        log::error!("affichage de la fenêtre a échoué : {e}");
    }
    layout::watch(window.clone());
    persistence::track_moves(window);
}

pub fn move_to_monitor(window: &WebviewWindow, monitor_key: &str) -> tauri::Result<()> {
    let prefs = window.state::<PreferencesStore>().snapshot();
    placement::move_to_monitor(window, &prefs, monitor_key)?;
    // Le nouvel écran peut déjà avoir une application en plein écran.
    foreground::refresh();
    Ok(())
}
