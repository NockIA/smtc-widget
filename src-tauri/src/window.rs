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
