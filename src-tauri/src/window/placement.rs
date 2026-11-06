use tauri::WebviewWindow;

use super::constants::{PRIMARY_SCREEN_RIGHT_MARGIN, SECONDARY_SCREEN_RIGHT_MARGIN};

/// Place la fenêtre en bas à droite de l'écran secondaire s'il existe, sinon de l'écran principal.
pub fn place_on_preferred_monitor(window: &WebviewWindow) -> tauri::Result<()> {
    let primary = window.primary_monitor()?;
    let monitors = window.available_monitors()?;

    let secondary = monitors.iter().find(|m| {
        primary
            .as_ref()
            .is_none_or(|p| m.position() != p.position())
    });

    let target = secondary.or(primary.as_ref()).or(monitors.first());

    let Some(monitor) = target else {
        return Ok(());
    };

    let right_margin = if secondary.is_some() {
        SECONDARY_SCREEN_RIGHT_MARGIN
    } else {
        PRIMARY_SCREEN_RIGHT_MARGIN
    };

    let size = window.outer_size()?;
    let margin = (right_margin * monitor.scale_factor()) as i32;
    let x = monitor.position().x + monitor.size().width as i32 - size.width as i32 - margin;
    let y = monitor.position().y + monitor.size().height as i32 - size.height as i32;

    window.set_position(tauri::PhysicalPosition::new(x, y))
}
