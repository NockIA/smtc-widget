mod constants;
mod persistence;
mod placement;
mod preferences;
mod topmost;

use std::path::{Path, PathBuf};

use tauri::WebviewWindow;

pub use placement::list_monitors;
pub use topmost::{keep_on_top, pause_keep_on_top};

pub fn preferences_path(config_dir: PathBuf) -> PathBuf {
    config_dir.join(constants::PREFERENCES_FILE)
}

pub fn init_placement(window: WebviewWindow, preferences_path: PathBuf) {
    let prefs = preferences::WindowPreferences::load(&preferences_path);

    if let Err(e) = placement::place_on_preferred_monitor(&window, &prefs) {
        eprintln!("placement de la fenêtre échoué : {e}");
    }
    persistence::track_moves(window, preferences_path, prefs);
}

pub fn move_to_monitor(
    window: &WebviewWindow,
    preferences_path: &Path,
    monitor_key: &str,
) -> tauri::Result<()> {
    let prefs = preferences::WindowPreferences::load(preferences_path);
    placement::move_to_monitor(window, &prefs, monitor_key)
}
