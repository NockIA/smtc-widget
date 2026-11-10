use tauri::{Monitor, PhysicalPosition, WebviewWindow};

use super::models::MonitorEntry;
use super::size::{physical_size, scale_from_width};
use super::taskbar_layout::right_cluster_left;
use crate::features::preferences::{DEFAULT_SCALE, MonitorPlacement, Preferences};

const DEFAULT_BOTTOM_MARGIN: f64 = 0.0;

/// Espace (pixels logiques) laissé entre le widget et les éléments de droite de la barre.
const TASKBAR_GAP: f64 = 8.0;

/// Marges de repli (pixels logiques) quand la barre des tâches n'est pas lisible.
const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;
const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

/// Identifiant d'un écran pour les préférences : nom + résolution.
pub fn monitor_key(monitor: &Monitor) -> String {
    format!(
        "{}@{}x{}",
        monitor.name().map_or("inconnu", String::as_str),
        monitor.size().width,
        monitor.size().height
    )
}

fn monitor_right(monitor: &Monitor) -> i32 {
    monitor.position().x + monitor.size().width as i32
}

fn monitor_bottom(monitor: &Monitor) -> i32 {
    monitor.position().y + monitor.size().height as i32
}

fn is_secondary(primary: Option<&Monitor>, monitor: &Monitor) -> bool {
    primary.is_none_or(|p| monitor.position() != p.position())
}

/// Position par défaut : juste à gauche de ce qui occupe la droite de la barre des tâches
/// (météo, zone de notification, horloge), sinon les marges fixes.
pub fn default_placement(window: &WebviewWindow, monitor: &Monitor) -> tauri::Result<MonitorPlacement> {
    let right_margin = if let Some(left) = right_cluster_left(monitor) {
        f64::from(monitor_right(monitor) - left) / monitor.scale_factor() + TASKBAR_GAP
    } else {
        let primary = window.primary_monitor()?;
        if is_secondary(primary.as_ref(), monitor) {
            SECONDARY_SCREEN_RIGHT_MARGIN
        } else {
            PRIMARY_SCREEN_RIGHT_MARGIN
        }
    };
    Ok(MonitorPlacement {
        right_margin,
        bottom_margin: DEFAULT_BOTTOM_MARGIN,
        scale: DEFAULT_SCALE,
    })
}

pub fn current_placement(window: &WebviewWindow, monitor: &Monitor) -> tauri::Result<MonitorPlacement> {
    let position = window.outer_position()?;
    let size = window.outer_size()?;
    let scale = monitor.scale_factor();

    let right = monitor_right(monitor) - (position.x + size.width as i32);
    let bottom = monitor_bottom(monitor) - (position.y + size.height as i32);

    Ok(MonitorPlacement {
        right_margin: f64::from(right) / scale,
        bottom_margin: f64::from(bottom) / scale,
        scale: scale_from_width(f64::from(size.width), scale),
    })
}

/// Échelle actuelle du widget (1.0 = taille de base).
pub fn current_scale(window: &WebviewWindow) -> tauri::Result<f64> {
    Ok(scale_from_width(
        f64::from(window.outer_size()?.width),
        window.scale_factor()?,
    ))
}

/// Applique l'échelle puis la position, en gardant le coin bas-droit ancré.
pub fn apply(window: &WebviewWindow, monitor: &Monitor, placement: MonitorPlacement) -> tauri::Result<()> {
    let dpi = monitor.scale_factor();
    let size = physical_size(placement.scale, dpi);
    let position = PhysicalPosition::new(
        monitor_right(monitor) - size.width as i32 - (placement.right_margin * dpi) as i32,
        // Une position enregistrée sous le bord de l'écran (avant la limite du déplacement) est ramenée sur l'écran.
        monitor_bottom(monitor) - size.height as i32 - (placement.bottom_margin.max(0.0) * dpi) as i32,
    );

    // Entrer d'abord sur l'écran cible : un changement de DPI peut redimensionner la fenêtre, la taille se règle ensuite.
    window.set_position(position)?;
    window.set_size(size)?;
    window.set_zoom(placement.scale)?;
    window.set_position(position)
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

/// Position sauvegardée pour cet écran, sinon celle par défaut.
fn placement_for(window: &WebviewWindow, prefs: &Preferences, monitor: &Monitor) -> tauri::Result<MonitorPlacement> {
    match prefs.monitors.get(&monitor_key(monitor)) {
        Some(saved) => Ok(*saved),
        None => default_placement(window, monitor),
    }
}

pub fn place_on_preferred_monitor(window: &WebviewWindow, prefs: &Preferences) -> tauri::Result<()> {
    let Some(monitor) = choose_monitor(window, prefs)? else {
        return Ok(());
    };

    apply(window, &monitor, placement_for(window, prefs, &monitor)?)
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

    apply(window, monitor, placement_for(window, prefs, monitor)?)
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
