use std::error::Error;

use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, WebviewWindow, Wry};

use crate::media::{self, MediaState};
use crate::settings::{self, SettingsState, ThemePreference};
use crate::window;

const QUIT_ID: &str = "quit";
const SOURCE_PREFIX: &str = "source:";
const MONITOR_PREFIX: &str = "monitor:";
const THEME_PREFIX: &str = "theme:";

fn build_menu(app: &AppHandle, window: &WebviewWindow) -> Result<Menu<Wry>, Box<dyn Error>> {
    let selected = app.state::<MediaState>().selected();
    let sources = media::list_sources(selected.as_deref())?;
    let source_menu = Submenu::new(app, "Piste audio", !sources.is_empty())?;
    for source in &sources {
        source_menu.append(&CheckMenuItem::with_id(
            app,
            format!("{SOURCE_PREFIX}{}", source.id),
            &source.label,
            true,
            source.active,
            None::<&str>,
        )?)?;
    }

    let monitors = window::list_monitors(window)?;
    let monitor_menu = Submenu::new(app, "Déplacer vers", monitors.len() > 1)?;
    for monitor in &monitors {
        monitor_menu.append(&CheckMenuItem::with_id(
            app,
            format!("{MONITOR_PREFIX}{}", monitor.key),
            &monitor.label,
            true,
            monitor.current,
            None::<&str>,
        )?)?;
    }

    let current_theme = app.state::<SettingsState>().theme();
    let theme_menu = Submenu::new(app, "Thème", true)?;
    for theme in ThemePreference::ALL {
        theme_menu.append(&CheckMenuItem::with_id(
            app,
            format!("{THEME_PREFIX}{}", theme.id()),
            theme.label(),
            true,
            theme == current_theme,
            None::<&str>,
        )?)?;
    }

    Ok(Menu::with_items(
        app,
        &[
            &source_menu,
            &monitor_menu,
            &theme_menu,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, QUIT_ID, "Quitter", true, None::<&str>)?,
        ],
    )?)
}

#[tauri::command]
pub async fn show_context_menu(
    app: AppHandle,
    window: WebviewWindow,
    x: f64,
) -> Result<(), String> {
    let menu = build_menu(&app, &window).map_err(|e| e.to_string())?;

    // Le widget est collé en bas de l'écran : le menu s'ouvre au-dessus de son bord haut,
    // à l'abscisse du clic, pour ne pas passer derrière la barre des tâches.
    let _pause = window::pause_keep_on_top();
    window
        .popup_menu_at(&menu, tauri::LogicalPosition::new(x, 0.0))
        .map_err(|e| e.to_string())
}

fn set_theme(app: &AppHandle, theme: ThemePreference) {
    if let Err(e) = app.state::<SettingsState>().set_theme(theme) {
        eprintln!("sauvegarde du thème échouée : {e}");
    }
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = settings::apply_theme(&window, theme) {
            eprintln!("application du thème échouée : {e}");
        }
    }
}

fn move_to_monitor(app: &AppHandle, key: &str) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let result = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())
        .and_then(|dir| {
            window::move_to_monitor(&window, &window::preferences_path(dir), key)
                .map_err(|e| e.to_string())
        });
    if let Err(e) = result {
        eprintln!("déplacement vers l'écran échoué : {e}");
    }
}

pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();

    if id == QUIT_ID {
        app.exit(0);
    } else if let Some(source) = id.strip_prefix(SOURCE_PREFIX) {
        app.state::<MediaState>().select_source(source.to_owned());
    } else if let Some(key) = id.strip_prefix(MONITOR_PREFIX) {
        move_to_monitor(app, key);
    } else if let Some(theme) = id.strip_prefix(THEME_PREFIX).and_then(ThemePreference::from_id) {
        set_theme(app, theme);
    }
}
