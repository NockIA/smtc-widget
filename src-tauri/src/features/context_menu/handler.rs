use tauri::menu::MenuEvent;
use tauri::{AppHandle, Manager};

use tauri_plugin_autostart::ManagerExt;

use super::action::MenuAction;
use crate::features::appearance;
use crate::features::audio_source::AudioSourceState;
use crate::features::preferences::{PreferencesStore, ThemePreference};
use crate::features::window;

fn set_theme(app: &AppHandle, theme: ThemePreference) {
    if let Err(e) = app.state::<PreferencesStore>().update(|prefs| prefs.theme = theme) {
        log::error!("sauvegarde du thème a échoué : {e}");
    }
    if let Some(window) = app.get_webview_window("main")
        && let Err(e) = appearance::apply_theme(&window, theme)
    {
        log::error!("application du thème a échoué : {e}");
    }
}

fn move_to_monitor(app: &AppHandle, key: &str) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(e) = window::move_to_monitor(&window, key) {
        log::error!("déplacement vers l'écran a échoué : {e}");
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
        log::error!("changement du lancement au démarrage a échoué : {e}");
    }
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let Some(action) = MenuAction::parse(event.id().as_ref()) else {
        return;
    };

    match action {
        MenuAction::Quit => app.exit(0),
        MenuAction::Autostart => toggle_autostart(app),
        MenuAction::Source(id) => app.state::<AudioSourceState>().select_source(id),
        MenuAction::Monitor(key) => move_to_monitor(app, &key),
        MenuAction::Theme(theme) => set_theme(app, theme),
    }
}
