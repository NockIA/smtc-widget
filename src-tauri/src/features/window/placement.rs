use tauri::{Monitor, PhysicalPosition, WebviewWindow};

use super::display_names::{get_display_number, get_friendly_names};
use super::models::MonitorEntry;
use super::size::{physical_size, scale_from_width};
use super::taskbar_layout::find_system_area_left_edge;
use crate::features::preferences::{DEFAULT_SCALE, MonitorPlacement, Preferences};

const DEFAULT_BOTTOM_MARGIN: f64 = 0.0;

/// Espace (pixels logiques) laissé entre le widget et les éléments de droite de la barre.
const TASKBAR_GAP: f64 = 8.0;

/// Marge droite de repli (pixels logiques) quand la barre des tâches n'est pas lisible.
const FALLBACK_RIGHT_MARGIN: f64 = 75.0;

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

fn is_primary(primary: Option<&Monitor>, monitor: &Monitor) -> bool {
    primary.is_some_and(|p| monitor.position() == p.position())
}

/// Position par défaut : juste à gauche de ce qui occupe la droite de la barre des tâches
/// (météo, zone de notification, horloge), sinon les marges fixes.
pub fn default_placement(monitor: &Monitor) -> MonitorPlacement {
    let right_margin = find_system_area_left_edge(monitor).map_or(FALLBACK_RIGHT_MARGIN, |left| {
        f64::from(monitor_right(monitor) - left) / monitor.scale_factor() + TASKBAR_GAP
    });
    MonitorPlacement {
        right_margin,
        bottom_margin: DEFAULT_BOTTOM_MARGIN,
        scale: DEFAULT_SCALE,
    }
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

    Ok(last_used.or(primary.as_ref()).or(monitors.first()).cloned())
}

/// Position sauvegardée pour cet écran, sinon celle par défaut.
fn placement_for(prefs: &Preferences, monitor: &Monitor) -> MonitorPlacement {
    prefs
        .monitors
        .get(&monitor_key(monitor))
        .copied()
        .unwrap_or_else(|| default_placement(monitor))
}

pub fn place_on_preferred_monitor(window: &WebviewWindow, prefs: &Preferences) -> tauri::Result<()> {
    let Some(monitor) = choose_monitor(window, prefs)? else {
        return Ok(());
    };

    apply(window, &monitor, placement_for(prefs, &monitor))
}

pub fn list_monitors(window: &WebviewWindow) -> tauri::Result<Vec<MonitorEntry>> {
    let primary = window.primary_monitor()?;
    let current = window.current_monitor()?.map(|m| monitor_key(&m));
    let friendly_names = get_friendly_names();

    // Numérotation de Paramètres › Affichage ; l'ordre d'énumération sert seulement de repli.
    let mut numbered: Vec<(u32, MonitorEntry)> = window
        .available_monitors()?
        .iter()
        .zip(1..)
        .map(|(monitor, fallback_number)| {
            let display_name = monitor.name().map_or("", String::as_str);
            let number = get_display_number(display_name).unwrap_or(fallback_number);
            let key = monitor_key(monitor);

            let mut parts = vec![format!("Écran {number}")];
            parts.extend(friendly_names.get(display_name).cloned());
            parts.push(format!("{}x{}", monitor.size().width, monitor.size().height));
            let suffix = if is_primary(primary.as_ref(), monitor) {
                " (principal)"
            } else {
                ""
            };

            let entry = MonitorEntry {
                label: format!("{}{suffix}", parts.join(" - ")),
                current: current.as_deref() == Some(key.as_str()),
                key,
            };
            (number, entry)
        })
        .collect();
    numbered.sort_by_key(|(number, _)| *number);

    Ok(numbered.into_iter().map(|(_, entry)| entry).collect())
}

/// Déplace la fenêtre sur l'écran `key`, à sa position sauvegardée ou par défaut.
pub fn move_to_monitor(window: &WebviewWindow, prefs: &Preferences, key: &str) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let Some(monitor) = monitors.iter().find(|m| monitor_key(m) == key) else {
        return Ok(());
    };

    apply(window, monitor, placement_for(prefs, monitor))
}

/// Empreinte de la disposition des écrans (noms, tailles, positions), pour détecter un changement.
pub fn compute_monitors_signature(window: &WebviewWindow) -> tauri::Result<String> {
    let mut parts: Vec<String> = window
        .available_monitors()?
        .iter()
        .map(|m| format!("{}@{},{}", monitor_key(m), m.position().x, m.position().y))
        .collect();
    parts.sort();
    Ok(parts.join("|"))
}
