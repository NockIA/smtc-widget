use tauri::{Manager, WebviewWindow};

use super::{bounds, foreground, layout, persistence, placement};
use crate::features::preferences::PreferencesStore;

/// Place la fenêtre selon les préférences, l'affiche, puis suit ses déplacements et les changements d'écrans.
pub fn init_placement(window: WebviewWindow) {
    let prefs = window.state::<PreferencesStore>().snapshot();

    bounds::install_bottom_limit(&window);
    if let Err(e) = placement::place_on_preferred_monitor(&window, &prefs) {
        log::error!("placement de la fenêtre a échoué : {e}");
    }
    // La fenêtre est créée invisible (`visible: false`)
    // on l'affiche seulement une fois placée, pour éviter un saut visible.
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
