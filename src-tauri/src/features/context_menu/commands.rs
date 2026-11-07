use tauri::{AppHandle, WebviewWindow};

use super::builder::build_menu;
use crate::features::window;
use crate::shared::error::AppResult;

#[tauri::command]
pub async fn show_context_menu(app: AppHandle, window: WebviewWindow, x: f64) -> AppResult<()> {
    let menu = build_menu(&app, &window)?;

    // Le widget est collé en bas de l'écran : le menu s'ouvre au-dessus de son bord haut,
    // à l'abscisse du clic, pour ne pas passer derrière la barre des tâches.
    let _pause = window::pause_keep_on_top();
    window.popup_menu_at(&menu, tauri::LogicalPosition::new(x, 0.0))?;
    Ok(())
}
