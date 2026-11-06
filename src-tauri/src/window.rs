use std::{thread, time::Duration};

use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Shell::{
            QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
            SHQueryUserNotificationState,
        },
        WindowsAndMessaging::{HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos},
    },
};

const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;
const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

/// Intervalle entre deux vérifications.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Vrai si une app occupe tout l'écran (vidéo, jeu, présentation).
fn is_fullscreen_app_running() -> bool {
    match unsafe { SHQueryUserNotificationState() } {
        Ok(state) => matches!(
            state,
            QUNS_BUSY | QUNS_PRESENTATION_MODE | QUNS_RUNNING_D3D_FULL_SCREEN
        ),
        // En cas d'erreur, on reste visible.
        Err(_) => false,
    }
}

/// Cache la fenêtre en plein écran, sinon la remet devant la barre des tâches.
pub fn keep_on_top(window: WebviewWindow) {
    thread::spawn(move || {
        // Le HWND n'est pas `Send` : on le récupère dans le thread.
        let hwnd: HWND = match window.hwnd() {
            Ok(hwnd) => hwnd,
            Err(e) => {
                eprintln!("impossible d'obtenir le HWND : {e}");
                return;
            }
        };

        let mut hidden = false;
        loop {
            if is_fullscreen_app_running() {
                if !hidden {
                    if let Err(e) = window.hide() {
                        eprintln!("hide a échoué : {e}");
                    }
                    hidden = true;
                }
            } else {
                if hidden {
                    if let Err(e) = window.show() {
                        eprintln!("show a échoué : {e}");
                    }
                    hidden = false;
                }

                // Change uniquement le z-order : ni position, ni taille, ni focus.
                let result = unsafe {
                    SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    )
                };
                if let Err(e) = result {
                    eprintln!("SetWindowPos a échoué : {e}");
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}

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
