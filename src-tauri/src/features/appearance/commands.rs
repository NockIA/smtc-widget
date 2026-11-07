use tauri::WebviewWindow;

use super::models::SystemColors;
use super::system_colors::{read_system_colors, request_refresh};
use crate::shared::error::AppResult;

#[tauri::command]
pub async fn get_system_colors(window: WebviewWindow) -> AppResult<SystemColors> {
    Ok(read_system_colors(&window)?)
}

/// Demande une relecture des couleurs (appelée au survol du widget).
#[tauri::command]
pub fn refresh_system_colors() {
    request_refresh();
}
