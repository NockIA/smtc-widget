use tauri::menu::MenuEvent;
use tauri::{AppHandle, Manager};

use tauri_plugin_autostart::ManagerExt;

use super::constants::{AUTOSTART_ID, MONITOR_PREFIX, QUIT_ID, SOURCE_PREFIX, THEME_PREFIX};
use crate::features::appearance;
use crate::features::audio_source::AudioSourceState;
use crate::features::preferences::{PreferencesStore, ThemePreference};
use crate::features::window;

fn set_theme(app: &AppHandle, theme: ThemePreference) {
    if let Err(e) = app
        .state::<PreferencesStore>()
        .update(|prefs| prefs.theme = theme)
    {
        log::error!("sauvegarde du thème échouée : {e}");
    }
    if let Some(window) = app.get_webview_window("main")
        && let Err(e) = appearance::apply_theme(&window, theme)
    {
        log::error!("application du thème échouée : {e}");
    }
}

fn move_to_monitor(app: &AppHandle, key: &str) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(e) = window::move_to_monitor(&window, key) {
        log::error!("déplacement vers l'écran échoué : {e}");
    }
}

fn toggle_autostart(app: &AppHandle) {
    let autolaunch = app.autolaunch();
    let result = if autolaunch.is_enabled().unwrap_or(false) {
        autolaunch.disable()
    } else {
        autolaunch.enable()
    };
    if let Err(e) = result {
        log::error!("changement du lancement au démarrage échoué : {e}");
    }
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();

    if id == QUIT_ID {
        app.exit(0);
    } else if id == AUTOSTART_ID {
        toggle_autostart(app);
    } else if let Some(source) = id.strip_prefix(SOURCE_PREFIX) {
        app.state::<AudioSourceState>()
            .select_source(source.to_owned());
    } else if let Some(key) = id.strip_prefix(MONITOR_PREFIX) {
        move_to_monitor(app, key);
    } else if let Some(theme) = id
        .strip_prefix(THEME_PREFIX)
        .and_then(ThemePreference::from_id)
    {
        set_theme(app, theme);
    }
}
