use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_autostart::ManagerExt;

use super::constants::{AUTOSTART_ID, MONITOR_PREFIX, QUIT_ID, SOURCE_PREFIX, THEME_PREFIX};
use crate::features::audio_source::{self, AudioSourceState};
use crate::features::preferences::{PreferencesStore, ThemePreference};
use crate::features::window;
use crate::shared::error::AppResult;

fn theme_label(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::System => "Système",
        ThemePreference::Light => "Clair",
        ThemePreference::Dark => "Sombre",
    }
}

fn source_submenu(app: &AppHandle) -> AppResult<Submenu<Wry>> {
    let selected = app.state::<AudioSourceState>().selected();
    let sources = audio_source::list_sources(selected.as_deref())?;

    let menu = Submenu::new(app, "Piste audio", !sources.is_empty())?;
    for source in &sources {
        menu.append(&CheckMenuItem::with_id(
            app,
            format!("{SOURCE_PREFIX}{}", source.id),
            &source.label,
            true,
            source.active,
            None::<&str>,
        )?)?;
    }
    Ok(menu)
}

fn monitor_submenu(app: &AppHandle, window: &WebviewWindow) -> AppResult<Submenu<Wry>> {
    let monitors = window::list_monitors(window)?;

    let menu = Submenu::new(app, "Déplacer vers", monitors.len() > 1)?;
    for monitor in &monitors {
        menu.append(&CheckMenuItem::with_id(
            app,
            format!("{MONITOR_PREFIX}{}", monitor.key),
            &monitor.label,
            true,
            monitor.current,
            None::<&str>,
        )?)?;
    }
    Ok(menu)
}

fn theme_submenu(app: &AppHandle) -> AppResult<Submenu<Wry>> {
    let current = app.state::<PreferencesStore>().snapshot().theme;

    let menu = Submenu::new(app, "Thème", true)?;
    for theme in ThemePreference::ALL {
        menu.append(&CheckMenuItem::with_id(
            app,
            format!("{THEME_PREFIX}{}", theme.id()),
            theme_label(theme),
            true,
            theme == current,
            None::<&str>,
        )?)?;
    }
    Ok(menu)
}

pub fn build_menu(app: &AppHandle, window: &WebviewWindow) -> AppResult<Menu<Wry>> {
    Ok(Menu::with_items(
        app,
        &[
            &source_submenu(app)?,
            &monitor_submenu(app, window)?,
            &theme_submenu(app)?,
            &PredefinedMenuItem::separator(app)?,
            &CheckMenuItem::with_id(
                app,
                AUTOSTART_ID,
                "Lancer au démarrage",
                true,
                app.autolaunch().is_enabled().unwrap_or(false),
                None::<&str>,
            )?,
            &MenuItem::with_id(app, QUIT_ID, "Quitter", true, None::<&str>)?,
        ],
    )?)
}
