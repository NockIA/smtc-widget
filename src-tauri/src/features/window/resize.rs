use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tauri::WebviewWindow;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos};

use super::foreground::is_hidden;
use super::size::{Layout, current, factor_from_width, physical_size, set_current};

/// Cadence de mise à jour pendant le glissé (~60 images par seconde).
const FRAME_INTERVAL: Duration = Duration::from_millis(16);

/// Garde-fou : un glissé n'est jamais aussi long, on ne reste pas bloqué si le relâchement est manqué.
const MAX_DRAG_DURATION: Duration = Duration::from_secs(30);

type ResizeResult = Result<(), Box<dyn Error>>;

static DRAGGING: AtomicBool = AtomicBool::new(false);

/// Libère le verrou de glissé à la fin du thread, quelle que soit la sortie.
struct DragGuard;

impl Drop for DragGuard {
    fn drop(&mut self) {
        DRAGGING.store(false, Ordering::Release);
    }
}

/// Poignée saisie. Le coin haut-gauche zoome tout le contenu (ancre : coin bas-droit) ;
/// les bords gauche et droit n'élargissent que la fenêtre, ce qui espace le contenu (ancre : bord opposé).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Handle {
    Corner,
    Left,
    Right,
}

impl Handle {
    /// Vrai si la fenêtre s'étend vers la gauche (bord droit fixe).
    fn grows_left(self) -> bool {
        !matches!(self, Self::Right)
    }
}

/// Fenêtre à redimensionner, avec ses bords d'ancrage à conserver.
struct Anchored {
    hwnd: HWND,
    handle: Handle,
    left: i32,
    right: i32,
    bottom: i32,
    dpi: f64,
}

impl Anchored {
    fn of(window: &WebviewWindow, handle: Handle) -> tauri::Result<Self> {
        let position = window.outer_position()?;
        let size = window.outer_size()?;
        Ok(Self {
            hwnd: window.hwnd()?,
            handle,
            left: position.x,
            right: position.x + size.width as i32,
            bottom: position.y + size.height as i32,
            dpi: window.scale_factor()?,
        })
    }

    /// Position, taille et zoom : la fenêtre grandit vers le haut et du côté de la poignée.
    fn resize(&self, window: &WebviewWindow, layout: Layout) -> ResizeResult {
        let size = physical_size(layout, self.dpi);
        let (width, height) = (size.width as i32, size.height as i32);
        let x = if self.handle.grows_left() {
            self.right - width
        } else {
            self.left
        };
        unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                x,
                self.bottom - height,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }
        window.set_zoom(layout.scale)?;
        set_current(layout);
        Ok(())
    }
}

fn is_left_button_down() -> bool {
    // Windows renvoie un nombre dont le premier bit (celui qui, dans un nombre signé, indique qu'il est négatif)
    // vaut 1 quand le bouton est enfoncé : un résultat négatif signifie donc « enfoncé ».
    unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON.0)) < 0 }
}

fn cursor_x() -> Option<i32> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some(point.x)
}

/// Suit le curseur tant que le bouton est enfoncé : l'éloigner du widget, côté de la poignée, l'agrandit.
fn drag(window: &WebviewWindow, handle: Handle) -> ResizeResult {
    let anchored = Anchored::of(window, handle)?;
    let start = current();
    let start_width = f64::from(window.outer_size()?.width);
    let Some(start_x) = cursor_x() else {
        return Ok(());
    };
    let started = Instant::now();

    while is_left_button_down() && started.elapsed() < MAX_DRAG_DURATION && !is_hidden() {
        if let Some(x) = cursor_x() {
            let delta = if handle.grows_left() { start_x - x } else { x - start_x };
            let width = start_width + f64::from(delta);
            let layout = match handle {
                Handle::Corner => Layout::new(factor_from_width(width, start.stretch, anchored.dpi), start.stretch),
                Handle::Left | Handle::Right => {
                    Layout::new(start.scale, factor_from_width(width, start.scale, anchored.dpi))
                }
            };
            anchored.resize(window, layout)?;
        }
        thread::sleep(FRAME_INTERVAL);
    }
    Ok(())
}

/// Démarre le redimensionnement à la poignée ; sans effet si un glissé est déjà en cours.
pub fn start(window: WebviewWindow, handle: Handle) {
    if DRAGGING.swap(true, Ordering::AcqRel) {
        return;
    }
    thread::spawn(move || {
        let _guard = DragGuard;
        if let Err(e) = drag(&window, handle) {
            log::error!("redimensionnement a échoué : {e}");
        }
    });
}

/// Remet le widget à sa taille de base.
pub fn reset(window: &WebviewWindow) {
    let result = Anchored::of(window, Handle::Corner)
        .map_err(Box::<dyn Error>::from)
        .and_then(|anchored| anchored.resize(window, Layout::default()));
    if let Err(e) = result {
        log::error!("réinitialisation de la taille a échoué : {e}");
    }
}
