use tauri::{Monitor, PhysicalPosition, WebviewWindow};

use super::constants::{
    DEFAULT_BOTTOM_MARGIN, PRIMARY_SCREEN_RIGHT_MARGIN, SECONDARY_SCREEN_RIGHT_MARGIN,
};
use super::preferences::{MonitorPlacement, WindowPreferences};

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

pub fn default_placement(
    window: &WebviewWindow,
    monitor: &Monitor,
) -> tauri::Result<MonitorPlacement> {
    let primary = window.primary_monitor()?;
    let right_margin = if is_secondary(primary.as_ref(), monitor) {
        SECONDARY_SCREEN_RIGHT_MARGIN
    } else {
        PRIMARY_SCREEN_RIGHT_MARGIN
    };
    Ok(MonitorPlacement {
        right_margin,
        bottom_margin: DEFAULT_BOTTOM_MARGIN,
    })
}

pub fn current_placement(
    window: &WebviewWindow,
    monitor: &Monitor,
) -> tauri::Result<MonitorPlacement> {
    let position = window.outer_position()?;
    let size = window.outer_size()?;
    let scale = monitor.scale_factor();

    let right =
        monitor.position().x + monitor.size().width as i32 - (position.x + size.width as i32);
    let bottom =
        monitor.position().y + monitor.size().height as i32 - (position.y + size.height as i32);

    Ok(MonitorPlacement {
        right_margin: right as f64 / scale,
        bottom_margin: bottom as f64 / scale,
    })
}

fn apply(
    window: &WebviewWindow,
    monitor: &Monitor,
    placement: MonitorPlacement,
) -> tauri::Result<()> {
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

fn choose_monitor(
    window: &WebviewWindow,
    prefs: &WindowPreferences,
) -> tauri::Result<Option<Monitor>> {
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

pub fn place_on_preferred_monitor(
    window: &WebviewWindow,
    prefs: &WindowPreferences,
) -> tauri::Result<()> {
    let Some(monitor) = choose_monitor(window, prefs)? else {
        return Ok(());
    };

    let placement = match prefs.monitors.get(&monitor_key(&monitor)) {
        Some(saved) => *saved,
        None => default_placement(window, &monitor)?,
    };
    apply(window, &monitor, placement)
}
