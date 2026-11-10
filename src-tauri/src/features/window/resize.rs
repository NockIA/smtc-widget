use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use tauri::WebviewWindow;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos};

use super::foreground::is_hidden;
use super::size::{clamp_scale, physical_size, scale_from_width};
use crate::features::preferences::DEFAULT_SCALE;

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

/// Fenêtre à redimensionner, avec son coin bas-droit (ancre) à conserver.
struct Anchored {
    hwnd: HWND,
    right: i32,
    bottom: i32,
    dpi: f64,
}

impl Anchored {
    fn of(window: &WebviewWindow) -> tauri::Result<Self> {
        let position = window.outer_position()?;
        let size = window.outer_size()?;
        Ok(Self {
            hwnd: window.hwnd()?,
            right: position.x + size.width as i32,
            bottom: position.y + size.height as i32,
            dpi: window.scale_factor()?,
        })
    }

    /// Position, taille et zoom : la fenêtre grandit vers la gauche et le haut, le contenu suit la même échelle.
    fn resize(&self, window: &WebviewWindow, scale: f64) -> ResizeResult {
        let size = physical_size(scale, self.dpi);
        let (width, height) = (size.width as i32, size.height as i32);
        unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                self.right - width,
                self.bottom - height,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }
        window.set_zoom(scale)?;
        Ok(())
    }
}

fn left_button_down() -> bool {
    // Bit de poids fort = bouton enfoncé.
    unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON.0)) < 0 }
}

fn cursor_x() -> Option<i32> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some(point.x)
}

/// Suit le curseur tant que le bouton est enfoncé : le déplacer vers la gauche agrandit le widget.
fn drag(window: &WebviewWindow) -> ResizeResult {
    let anchored = Anchored::of(window)?;
    let start_width = f64::from(window.outer_size()?.width);
    let Some(start_x) = cursor_x() else {
        return Ok(());
    };
    let started = Instant::now();

    while left_button_down() && started.elapsed() < MAX_DRAG_DURATION && !is_hidden() {
        if let Some(x) = cursor_x() {
            let width = start_width + f64::from(start_x - x);
            anchored.resize(window, clamp_scale(scale_from_width(width, anchored.dpi)))?;
        }
        thread::sleep(FRAME_INTERVAL);
    }
    Ok(())
}

/// Démarre le redimensionnement à la poignée ; sans effet si un glissé est déjà en cours.
pub fn start(window: WebviewWindow) {
    if DRAGGING.swap(true, Ordering::AcqRel) {
        return;
    }
    thread::spawn(move || {
        let _guard = DragGuard;
        if let Err(e) = drag(&window) {
            log::error!("redimensionnement a échoué : {e}");
        }
    });
}

/// Remet le widget à sa taille de base.
pub fn reset(window: &WebviewWindow) {
    let result = Anchored::of(window)
        .map_err(Box::<dyn Error>::from)
        .and_then(|anchored| anchored.resize(window, DEFAULT_SCALE));
    if let Err(e) = result {
        log::error!("réinitialisation de la taille a échoué : {e}");
    }
}
