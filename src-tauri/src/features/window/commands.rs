use tauri::WebviewWindow;

use super::resize::{self, Handle};

/// Début d'un glissé de la poignée : le redimensionnement suit le curseur tant que le bouton gauche est enfoncé.
#[tauri::command]
pub fn start_resize(window: WebviewWindow, handle: Handle) {
    resize::start(window, handle);
}

#[tauri::command]
pub fn reset_size(window: WebviewWindow) {
    resize::reset(&window);
}
