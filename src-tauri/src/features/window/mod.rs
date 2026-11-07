mod constants;
mod layout;
mod models;
mod persistence;
mod placement;
mod topmost;
mod webview_memory;

use tauri::{Manager, WebviewWindow};

use crate::features::preferences::PreferencesStore;

pub use placement::list_monitors;
pub use topmost::{is_hidden, keep_on_top, pause_keep_on_top};

/// Place la fenêtre selon les préférences, puis suit ses déplacements et les changements d'écrans.
pub fn init_placement(window: WebviewWindow) {
    let prefs = window.state::<PreferencesStore>().snapshot();

    if let Err(e) = placement::place_on_preferred_monitor(&window, &prefs) {
        eprintln!("placement de la fenêtre échoué : {e}");
    }
    layout::watch(window.clone());
    persistence::track_moves(window);
}

pub fn move_to_monitor(window: &WebviewWindow, monitor_key: &str) -> tauri::Result<()> {
    let prefs = window.state::<PreferencesStore>().snapshot();
    placement::move_to_monitor(window, &prefs, monitor_key)
}
