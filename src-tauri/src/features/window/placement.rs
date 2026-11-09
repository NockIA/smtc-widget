use tauri::{Monitor, PhysicalPosition, WebviewWindow};

use super::constants::{
    DEFAULT_BOTTOM_MARGIN, PRIMARY_SCREEN_RIGHT_MARGIN, SECONDARY_SCREEN_RIGHT_MARGIN, TASKBAR_GAP,
};
use super::models::MonitorEntry;
use super::taskbar_layout::right_cluster_left;
use crate::features::preferences::{MonitorPlacement, Preferences};

/// Identifiant d'un écran pour les préférences : nom + résolution.
pub fn monitor_key(monitor: &Monitor) -> String {
    format!(
        "{}@{}x{}",
        monitor.name().map(String::as_str).unwrap_or("inconnu"),
        monitor.size().width,
        monitor.size().height
    )
}

fn is_secondary(primary: Option<&Monitor>, monitor: &Monitor) -> bool {
    primary.is_none_or(|p| monitor.position() != p.position())
}

/// Position par défaut : juste à gauche de ce qui occupe la droite de la barre des tâches
/// (météo, zone de notification, horloge), sinon les marges fixes.
pub fn default_placement(window: &WebviewWindow, monitor: &Monitor) -> tauri::Result<MonitorPlacement> {
    let monitor_right = monitor.position().x + monitor.size().width as i32;
    let right_margin = match right_cluster_left(monitor) {
        Some(left) => (monitor_right - left) as f64 / monitor.scale_factor() + TASKBAR_GAP,
        None => {
            let primary = window.primary_monitor()?;
            if is_secondary(primary.as_ref(), monitor) {
                SECONDARY_SCREEN_RIGHT_MARGIN
            } else {
                PRIMARY_SCREEN_RIGHT_MARGIN
            }
        }
    };
    Ok(MonitorPlacement {
        right_margin,
        bottom_margin: DEFAULT_BOTTOM_MARGIN,
    })
}

pub fn current_placement(window: &WebviewWindow, monitor: &Monitor) -> tauri::Result<MonitorPlacement> {
    let position = window.outer_position()?;
    let size = window.outer_size()?;
    let scale = monitor.scale_factor();

    let right = monitor.position().x + monitor.size().width as i32 - (position.x + size.width as i32);
    let bottom = monitor.position().y + monitor.size().height as i32 - (position.y + size.height as i32);

    Ok(MonitorPlacement {
        right_margin: right as f64 / scale,
        bottom_margin: bottom as f64 / scale,
    })
}

fn apply(window: &WebviewWindow, monitor: &Monitor, placement: MonitorPlacement) -> tauri::Result<()> {
    let size = window.outer_size()?;
    let scale = monitor.scale_factor();
    let x = monitor.position().x + monitor.size().width as i32
        - size.width as i32
        - (placement.right_margin * scale) as i32;
    let y = monitor.position().y + monitor.size().height as i32
        - size.height as i32
        - (placement.bottom_margin * scale) as i32;

    window.set_position(PhysicalPosition::new(x, y))
}

fn choose_monitor(window: &WebviewWindow, prefs: &Preferences) -> tauri::Result<Option<Monitor>> {
    let primary = window.primary_monitor()?;
    let monitors = window.available_monitors()?;

    let last_used = prefs
        .last_monitor
        .as_ref()
        .and_then(|key| monitors.iter().find(|m| monitor_key(m) == *key));
    let secondary = monitors.iter().find(|m| is_secondary(primary.as_ref(), m));

    Ok(last_used
        .or(secondary)
        .or(primary.as_ref())
        .or(monitors.first())
        .cloned())
}

pub fn place_on_preferred_monitor(window: &WebviewWindow, prefs: &Preferences) -> tauri::Result<()> {
    let Some(monitor) = choose_monitor(window, prefs)? else {
        return Ok(());
    };

    let placement = match prefs.monitors.get(&monitor_key(&monitor)) {
        Some(saved) => *saved,
        None => default_placement(window, &monitor)?,
    };
    apply(window, &monitor, placement)
}

pub fn list_monitors(window: &WebviewWindow) -> tauri::Result<Vec<MonitorEntry>> {
    let primary = window.primary_monitor()?;
    let current = window.current_monitor()?.map(|m| monitor_key(&m));

    Ok(window
        .available_monitors()?
        .iter()
        .enumerate()
        .map(|(index, monitor)| {
            let key = monitor_key(monitor);
            let suffix = if is_secondary(primary.as_ref(), monitor) {
                ""
            } else {
                " (principal)"
            };
            MonitorEntry {
                label: format!(
                    "Écran {} - {}x{}{}",
                    index + 1,
                    monitor.size().width,
                    monitor.size().height,
                    suffix
                ),
                current: current.as_deref() == Some(key.as_str()),
                key,
            }
        })
        .collect())
}

/// Déplace la fenêtre sur l'écran `key`, à sa position sauvegardée ou par défaut.
pub fn move_to_monitor(window: &WebviewWindow, prefs: &Preferences, key: &str) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let Some(monitor) = monitors.iter().find(|m| monitor_key(m) == key) else {
        return Ok(());
    };

    let placement = match prefs.monitors.get(key) {
        Some(saved) => *saved,
        None => default_placement(window, monitor)?,
    };
    apply(window, monitor, placement)
}

/// Empreinte de la disposition des écrans (noms, tailles, positions), pour détecter un changement.
pub fn monitors_signature(window: &WebviewWindow) -> tauri::Result<String> {
    let mut parts: Vec<String> = window
        .available_monitors()?
        .iter()
        .map(|m| format!("{}@{},{}", monitor_key(m), m.position().x, m.position().y))
        .collect();
    parts.sort();
    Ok(parts.join("|"))
}
