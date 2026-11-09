use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_autostart::ManagerExt;

use super::action::MenuAction;
use crate::features::audio_source::{self, AudioSourceState};
use crate::features::preferences::{PreferencesStore, ThemePreference};
use crate::features::window;
use crate::shared::error::{AppError, AppResult};

fn theme_label(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::System => "Système",
        ThemePreference::Light => "Clair",
        ThemePreference::Dark => "Sombre",
    }
}

fn check_item(app: &AppHandle, action: &MenuAction, label: &str, checked: bool) -> tauri::Result<CheckMenuItem<Wry>> {
    CheckMenuItem::with_id(app, action.id(), label, true, checked, None::<&str>)
}

fn source_submenu(app: &AppHandle) -> AppResult<Submenu<Wry>> {
    let selected = app.state::<AudioSourceState>().selected();
    let sources = audio_source::list_sources(selected.as_deref()).map_err(AppError::Smtc)?;

    let menu = Submenu::new(app, "Piste audio", !sources.is_empty())?;
    for source in sources {
        let action = MenuAction::Source(source.id);
        menu.append(&check_item(app, &action, &source.label, source.active)?)?;
    }
    Ok(menu)
}

fn monitor_submenu(app: &AppHandle, window: &WebviewWindow) -> AppResult<Submenu<Wry>> {
    let monitors = window::list_monitors(window)?;

    let menu = Submenu::new(app, "Déplacer vers", monitors.len() > 1)?;
    for monitor in monitors {
        let action = MenuAction::Monitor(monitor.key);
        menu.append(&check_item(app, &action, &monitor.label, monitor.current)?)?;
    }
    Ok(menu)
}

fn theme_submenu(app: &AppHandle) -> AppResult<Submenu<Wry>> {
    let current = app.state::<PreferencesStore>().snapshot().theme;

    let menu = Submenu::new(app, "Thème", true)?;
    for theme in ThemePreference::ALL {
        let action = MenuAction::Theme(theme);
        menu.append(&check_item(app, &action, theme_label(theme), theme == current)?)?;
    }
    Ok(menu)
}

pub fn build_menu(app: &AppHandle, window: &WebviewWindow) -> AppResult<Menu<Wry>> {
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    Ok(Menu::with_items(
        app,
        &[
            &source_submenu(app)?,
            &monitor_submenu(app, window)?,
            &theme_submenu(app)?,
            &PredefinedMenuItem::separator(app)?,
            &check_item(app, &MenuAction::Autostart, "Lancer au démarrage", autostart_enabled)?,
            &MenuItem::with_id(app, MenuAction::Quit.id(), "Quitter", true, None::<&str>)?,
        ],
    )?)
}
