use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;

use tauri::{Emitter, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent};
use windows::Foundation::TypedEventHandler;
use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
};

use super::constants::{
    FALLBACK_INTERVAL, MIN_SAMPLES, ROW_OFFSET_PX, SYSTEM_COLORS_EVENT, TRIGGER_DEBOUNCE,
    WINDOW_MARGIN_PX,
};
use super::models::SystemColors;
use crate::features::window;

type Rgb = [u8; 3];

/// Déclencheur partagé : relecture des couleurs à la demande (survol, déplacement, thème).
static TRIGGER: OnceLock<Sender<()>> = OnceLock::new();

pub fn request_refresh() {
    if let Some(trigger) = TRIGGER.get() {
        let _ = trigger.send(());
    }
}

fn hex([r, g, b]: Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn inside(position: &PhysicalPosition<i32>, size: &PhysicalSize<u32>, x: i32, y: i32) -> bool {
    x >= position.x
        && y >= position.y
        && x < position.x + size.width as i32
        && y < position.y + size.height as i32
}

/// La barre des tâches occupe la différence entre l'écran et sa zone de travail.
fn on_taskbar(monitor: &Monitor, x: i32, y: i32) -> bool {
    let work = monitor.work_area();
    inside(monitor.position(), monitor.size(), x, y) && !inside(&work.position, &work.size, x, y)
}

/// Capture une ligne d'écran de 1 px de haut en une seule copie (au lieu d'un `GetPixel` par point).
fn capture_row(left: i32, y: i32, width: i32) -> Option<Vec<Rgb>> {
    if width <= 0 {
        return None;
    }
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        let memory = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, width, 1);
        let previous = SelectObject(memory, bitmap.into());

        let copied = BitBlt(memory, 0, 0, width, 1, Some(screen), left, y, SRCCOPY).is_ok();
        // `GetDIBits` demande que le bitmap ne soit plus sélectionné dans un DC.
        SelectObject(memory, previous);

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -1,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buffer = vec![0u8; width as usize * 4];
        let lines = GetDIBits(
            memory,
            bitmap,
            0,
            1,
            Some(buffer.as_mut_ptr() as *mut c_void),
            &mut info,
            DIB_RGB_COLORS,
        );

        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory);
        ReleaseDC(None, screen);

        if !copied || lines == 0 {
            return None;
        }
        // Pixels au format BGRA.
        Some(
            buffer
                .as_chunks::<4>()
                .0
                .iter()
                .map(|p| [p[2], p[1], p[0]])
                .collect(),
        )
    }
}

/// Médiane par canal : insensible aux icônes (minoritaires) et au grain de l'acrylique.
fn median(pixels: &[Rgb]) -> Rgb {
    let channel = |index: usize| {
        let mut values: Vec<u8> = pixels.iter().map(|p| p[index]).collect();
        values.sort_unstable();
        values[values.len() / 2]
    };
    [channel(0), channel(1), channel(2)]
}

/// Couleur réelle de la barre des tâches, lue à l'écran à côté du widget.
/// `None` si le widget n'est pas posé sur la barre (ou si elle est masquée).
fn taskbar_color(window: &WebviewWindow) -> tauri::Result<Option<String>> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(None);
    };
    let position = window.outer_position()?;
    let size = window.outer_size()?;

    let center_x = position.x + size.width as i32 / 2;
    let center_y = position.y + size.height as i32 / 2;
    if !on_taskbar(&monitor, center_x, center_y) {
        return Ok(None);
    }

    let left = monitor.position().x;
    let y = position.y + ROW_OFFSET_PX;
    let Some(row) = capture_row(left, y, monitor.size().width as i32) else {
        return Ok(None);
    };

    let pixels: Vec<Rgb> = row
        .into_iter()
        .enumerate()
        .map(|(offset, pixel)| (left + offset as i32, pixel))
        .filter(|&(x, _)| {
            x < position.x - WINDOW_MARGIN_PX
                || x > position.x + size.width as i32 + WINDOW_MARGIN_PX
        })
        .filter(|&(x, _)| on_taskbar(&monitor, x, y))
        .map(|(_, pixel)| pixel)
        .collect();

    if pixels.len() < MIN_SAMPLES {
        return Ok(None);
    }
    Ok(Some(hex(median(&pixels))))
}

/// Couleur d'accentuation choisie dans les paramètres de Windows.
fn accent_color(settings: &UISettings) -> Option<String> {
    let color = settings.GetColorValue(UIColorType::Accent).ok()?;
    Some(hex([color.R, color.G, color.B]))
}

fn read_with(window: &WebviewWindow, settings: Option<&UISettings>) -> tauri::Result<SystemColors> {
    Ok(SystemColors {
        taskbar: taskbar_color(window)?,
        accent: settings.and_then(accent_color),
    })
}

pub(super) fn read_system_colors(window: &WebviewWindow) -> tauri::Result<SystemColors> {
    read_with(window, UISettings::new().ok().as_ref())
}

/// Relit les couleurs sur déclencheur (thème/accent, fin de déplacement, survol),
/// avec un contrôle de secours lent. Rien n'est lu tant que le widget est caché.
pub fn watch(window: WebviewWindow) {
    let (tx, rx) = mpsc::channel::<()>();
    let _ = TRIGGER.set(tx.clone());

    let moved = tx.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Moved(_) = event {
            let _ = moved.send(());
        }
    });

    thread::spawn(move || {
        // Gardé vivant dans le thread pour conserver l'abonnement aux changements de couleurs.
        let settings = UISettings::new().ok();
        if let Some(settings) = &settings {
            let changed = tx.clone();
            let handler = TypedEventHandler::new(move |_, _| {
                let _ = changed.send(());
                Ok(())
            });
            if let Err(e) = settings.ColorValuesChanged(&handler) {
                log::error!("abonnement aux couleurs Windows échoué : {e}");
            }
        }

        let mut last: Option<SystemColors> = None;
        loop {
            if !window::is_hidden() {
                match read_with(&window, settings.as_ref()) {
                    Ok(colors) if last.as_ref() != Some(&colors) => {
                        if let Err(e) = window.emit(SYSTEM_COLORS_EVENT, &colors) {
                            log::error!("emit {SYSTEM_COLORS_EVENT} a échoué : {e}");
                        }
                        last = Some(colors);
                    }
                    Ok(_) => {}
                    Err(e) => log::error!("lecture des couleurs système échouée : {e}"),
                }
            }

            match rx.recv_timeout(FALLBACK_INTERVAL) {
                // Regroupe les rafales (un déplacement émet beaucoup d'événements).
                Ok(()) => while rx.recv_timeout(TRIGGER_DEBOUNCE).is_ok() {},
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}
