use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;

use tauri::{Emitter, WebviewWindow};
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook};
use windows::Win32::UI::Shell::{
    QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EVENT_SYSTEM_FOREGROUND, EnumWindows, GWL_EXSTYLE, GetMessageW, GetWindowLongPtrW, HWND_TOPMOST,
    IsIconic, IsWindowVisible, MSG, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetTimer, SetWindowPos, TranslateMessage,
    WINEVENT_OUTOFCONTEXT, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
};
use windows::core::BOOL;

use super::webview_memory;
use super::win32::{class_name, is_cloaked, window_rect};

/// Émis vers le front quand le widget est caché (`true`) ou réaffiché (`false`).
const VISIBILITY_EVENT: &str = "widget-visibility";

/// Contrôle de secours du premier plan, en plus des changements de fenêtre active.
const TOPMOST_FALLBACK_INTERVAL_MS: u32 = 2_000;

/// Fenêtres du bureau et de la barre des tâches : elles couvrent l'écran sans être des applications plein écran.
const SHELL_CLASSES: [&str; 4] = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

static WIDGET: OnceLock<WebviewWindow> = OnceLock::new();
/// Lu une fois au démarrage : `window.hwnd()` passe par le thread principal. Valeur brute car HWND n'est pas `Send`.
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

/// Windows signale les jeux Direct3D exclusifs et le mode présentation, mais pas toujours une vidéo de navigateur.
fn notification_state_is_fullscreen() -> bool {
    match unsafe { SHQueryUserNotificationState() } {
        Ok(state) => matches!(state, QUNS_BUSY | QUNS_PRESENTATION_MODE | QUNS_RUNNING_D3D_FULL_SCREEN),
        // En cas d'erreur, on reste visible.
        Err(_) => false,
    }
}

/// Fenêtre candidate et écran du widget, partagés avec le callback d'`EnumWindows`.
struct FullscreenSearch {
    widget: HWND,
    screen: RECT,
    found: bool,
}

impl FullscreenSearch {
    /// Vrai si `window` est une application visible qui recouvre tout l'écran du widget.
    fn is_fullscreen(&self, window: HWND) -> bool {
        // Les tests les moins coûteux d'abord : cette fonction passe sur toutes les fenêtres du système.
        if window == self.widget
            || !unsafe { IsWindowVisible(window) }.as_bool()
            || unsafe { IsIconic(window) }.as_bool()
        {
            return false;
        }
        let Some(rect) = window_rect(window) else {
            return false;
        };
        let covers_screen = rect.left <= self.screen.left
            && rect.top <= self.screen.top
            && rect.right >= self.screen.right
            && rect.bottom >= self.screen.bottom;
        if !covers_screen {
            return false;
        }

        // Les surcouches (Game Bar, enregistreurs d'écran…) couvrent l'écran sans le monopoliser : elles laissent passer les clics.
        let style = unsafe { GetWindowLongPtrW(window, GWL_EXSTYLE) } as u32;
        let click_through = style & (WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0) != 0;
        !click_through && !is_cloaked(window) && !SHELL_CLASSES.contains(&class_name(window).as_str())
    }
}

unsafe extern "system" fn check_window(window: HWND, search: LPARAM) -> BOOL {
    let search = unsafe { &mut *(search.0 as *mut FullscreenSearch) };
    search.found = search.is_fullscreen(window);
    // Arrête l'énumération dès qu'une fenêtre plein écran est trouvée.
    (!search.found).into()
}

/// Vrai si une application recouvre tout l'écran où se trouve le widget, qu'elle soit active ou non
/// (par exemple juste après avoir déplacé le widget sur un écran où une vidéo est déjà en plein écran).
fn fullscreen_window_on_widget_screen() -> bool {
    let widget = HWND(WIDGET_HWND.load(Ordering::Relaxed) as *mut _);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let has_info = unsafe { GetMonitorInfoW(MonitorFromWindow(widget, MONITOR_DEFAULTTONEAREST), &mut info) };
    if !has_info.as_bool() {
        return false;
    }

    let mut search = FullscreenSearch {
        widget,
        screen: info.rcMonitor,
        found: false,
    };
    // Une énumération interrompue par le callback renvoie une erreur : ce n'est pas un échec.
    let _ = unsafe { EnumWindows(Some(check_window), LPARAM(&raw mut search as isize)) };
    search.found
}

/// Ce qui indique qu'une application plein écran recouvre le widget, `None` sinon.
fn fullscreen_cause() -> Option<&'static str> {
    if notification_state_is_fullscreen() {
        Some("état de notification Windows")
    } else if fullscreen_window_on_widget_screen() {
        Some("fenêtre plein écran sur l'écran du widget")
    } else {
        None
    }
}

fn set_hidden(window: &WebviewWindow, hidden: bool) {
    let result = if hidden { window.hide() } else { window.show() };
    if let Err(e) = result {
        log::error!("changement de visibilité a échoué : {e}");
    }
    HIDDEN.store(hidden, Ordering::Relaxed);
    webview_memory::set_low_memory(window, hidden);
    if let Err(e) = window.emit(VISIBILITY_EVENT, hidden) {
        log::error!("emit {VISIBILITY_EVENT} a échoué : {e}");
    }
}

/// Cache ou réaffiche la fenêtre selon la présence d'une application plein écran ; renvoie `true` si elle est cachée.
fn sync_visibility(window: &WebviewWindow) -> bool {
    let cause = fullscreen_cause();
    let fullscreen = cause.is_some();
    if fullscreen != is_hidden() {
        log::debug!(
            "widget {} ({})",
            if fullscreen { "caché" } else { "réaffiché" },
            cause.unwrap_or("plus de plein écran")
        );
        set_hidden(window, fullscreen);
    }
    fullscreen
}

/// Remet la fenêtre devant la barre des tâches : change uniquement le z-order, ni position, ni taille, ni focus.
fn raise_to_top() {
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
        log::error!("SetWindowPos a échoué : {e}");
    }
}

/// Réévalue tout de suite visibilité et premier plan (déplacement du widget, fin d'un menu…).
pub fn refresh() {
    if PAUSED.load(Ordering::Relaxed) {
        return;
    }
    let Some(window) = WIDGET.get() else {
        return;
    };
    if !sync_visibility(window) {
        raise_to_top();
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
            log::error!("impossible d'obtenir le HWND : {e}");
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
            log::error!("SetWinEventHook a échoué");
        }
        SetTimer(None, 0, TOPMOST_FALLBACK_INTERVAL_MS, Some(on_fallback_timer));

        refresh();

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    });
}
