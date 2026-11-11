use tauri::WebviewWindow;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromRect};
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::WM_MOVING;

use super::win32::get_monitor_rect;

/// Identifiant de ce sous-classement parmi ceux de la fenêtre.
const SUBCLASS_ID: usize = 1;

/// Remonte le rectangle proposé pour qu'il ne dépasse pas le bas de son écran.
fn keep_above_screen_bottom(rect: &mut RECT) {
    let Some(screen) = get_monitor_rect(unsafe { MonitorFromRect(rect, MONITOR_DEFAULTTONEAREST) }) else {
        return;
    };
    let overflow = rect.bottom - screen.bottom;
    if overflow > 0 {
        rect.top -= overflow;
        rect.bottom -= overflow;
    }
}

/// Reçoit les messages de la fenêtre avant son traitement habituel.
unsafe extern "system" fn on_message(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _data: usize,
) -> LRESULT {
    if message == WM_MOVING {
        // Pendant un déplacement à la souris, Windows envoie la position qu'il va donner à la fenêtre
        // sous forme d'un rectangle (`lparam`). Le modifier ici change l'endroit où la fenêtre est réellement placée.
        keep_above_screen_bottom(unsafe { &mut *(lparam.0 as *mut RECT) });
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

/// Empêche de faire glisser le widget sous le bord bas de l'écran.
pub fn install_bottom_limit(window: &WebviewWindow) {
    let hwnd = match window.hwnd() {
        Ok(hwnd) => hwnd,
        Err(e) => {
            log::error!("impossible d'obtenir le HWND : {e}");
            return;
        }
    };
    if !unsafe { SetWindowSubclass(hwnd, Some(on_message), SUBCLASS_ID, 0) }.as_bool() {
        log::error!("limite basse du déplacement non installée");
    }
}
