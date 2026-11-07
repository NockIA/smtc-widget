use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;

use tauri::{Emitter, WebviewWindow};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook};
use windows::Win32::UI::Shell::{
    QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EVENT_SYSTEM_FOREGROUND, GetMessageW, HWND_TOPMOST, MSG, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SetTimer, SetWindowPos, TranslateMessage, WINEVENT_OUTOFCONTEXT,
};

use super::constants::{TOPMOST_FALLBACK_INTERVAL_MS, VISIBILITY_EVENT};
use super::webview_memory;

static WIDGET: OnceLock<WebviewWindow> = OnceLock::new();
/// Le HWND n'est pas `Send` : on garde sa valeur brute.
static WIDGET_HWND: AtomicIsize = AtomicIsize::new(0);
static HIDDEN: AtomicBool = AtomicBool::new(false);
/// Tant que c'est vrai, la fenêtre n'est plus remise au premier plan : sinon elle recouvre le menu contextuel.
static PAUSED: AtomicBool = AtomicBool::new(false);

pub struct PauseGuard;

impl Drop for PauseGuard {
    fn drop(&mut self) {
        PAUSED.store(false, Ordering::Relaxed);
        refresh();
    }
}

/// Suspend le maintien au premier plan jusqu'à la destruction du garde.
pub fn pause_keep_on_top() -> PauseGuard {
    PAUSED.store(true, Ordering::Relaxed);
    PauseGuard
}

/// Vrai pendant qu'une application plein écran masque le widget.
pub fn is_hidden() -> bool {
    HIDDEN.load(Ordering::Relaxed)
}

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

fn set_hidden(window: &WebviewWindow, hidden: bool) {
    let result = if hidden { window.hide() } else { window.show() };
    if let Err(e) = result {
        eprintln!("changement de visibilité échoué : {e}");
    }
    HIDDEN.store(hidden, Ordering::Relaxed);
    webview_memory::set_low_memory(window, hidden);
    if let Err(e) = window.emit(VISIBILITY_EVENT, hidden) {
        eprintln!("emit {VISIBILITY_EVENT} a échoué : {e}");
    }
}

/// Cache la fenêtre en plein écran, sinon la remet devant la barre des tâches.
fn refresh() {
    if PAUSED.load(Ordering::Relaxed) {
        return;
    }
    let Some(window) = WIDGET.get() else {
        return;
    };

    let fullscreen = is_fullscreen_app_running();
    if fullscreen != is_hidden() {
        set_hidden(window, fullscreen);
    }
    if fullscreen {
        return;
    }

    // Change uniquement le z-order : ni position, ni taille, ni focus.
    let hwnd = HWND(WIDGET_HWND.load(Ordering::Relaxed) as *mut _);
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

unsafe extern "system" fn on_foreground_changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    refresh();
}

unsafe extern "system" fn on_fallback_timer(_hwnd: HWND, _msg: u32, _id: usize, _time: u32) {
    refresh();
}

/// Réagit aux changements de fenêtre active (clic sur la barre des tâches, lancement d'un jeu…),
/// avec un contrôle de secours lent pour les cas sans changement de fenêtre.
pub fn keep_on_top(window: WebviewWindow) {
    match window.hwnd() {
        Ok(hwnd) => WIDGET_HWND.store(hwnd.0 as isize, Ordering::Relaxed),
        Err(e) => {
            eprintln!("impossible d'obtenir le HWND : {e}");
            return;
        }
    }
    if WIDGET.set(window).is_err() {
        return;
    }

    thread::spawn(|| unsafe {
        // Les callbacks sont appelés sur ce thread, par sa boucle de messages.
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(on_foreground_changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if hook.is_invalid() {
            eprintln!("SetWinEventHook a échoué");
        }
        SetTimer(
            None,
            0,
            TOPMOST_FALLBACK_INTERVAL_MS,
            Some(on_fallback_timer),
        );

        refresh();

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    });
}
