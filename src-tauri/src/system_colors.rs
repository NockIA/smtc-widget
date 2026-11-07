use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{Emitter, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};
use windows::UI::ViewManagement::{UIColorType, UISettings};
use windows::Win32::Graphics::Gdi::{CLR_INVALID, GetDC, GetPixel, ReleaseDC};

const SYSTEM_COLORS_EVENT: &str = "system-colors";
const POLL_INTERVAL: Duration = Duration::from_secs(3);

const SAMPLE_COUNT: i32 = 48;
const MIN_SAMPLES: usize = 3;
/// Distance minimale entre un point d'échantillonnage et le widget, qui recouvre la barre.
const WINDOW_MARGIN_PX: i32 = 8;
/// Les points sont pris sur une ligne proche du bord de la barre, là où il y a le moins d'icônes.
const ROW_OFFSET_PX: i32 = 3;

type Rgb = [u8; 3];

/// Couleurs de Windows (`#rrggbb`), `None` quand elles ne peuvent pas être lues.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemColors {
    pub taskbar: Option<String>,
    pub accent: Option<String>,
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

fn read_pixels(xs: impl Iterator<Item = i32>, y: i32) -> Vec<Rgb> {
    let mut pixels = Vec::new();
    unsafe {
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            return pixels;
        }
        for x in xs {
            let color = GetPixel(hdc, x, y).0;
            if color != CLR_INVALID {
                pixels.push([
                    (color & 0xFF) as u8,
                    ((color >> 8) & 0xFF) as u8,
                    ((color >> 16) & 0xFF) as u8,
                ]);
            }
        }
        ReleaseDC(None, hdc);
    }
    pixels
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

    let y = position.y + ROW_OFFSET_PX;
    let step = (monitor.size().width as i32 / SAMPLE_COUNT).max(1);
    let xs = (0..SAMPLE_COUNT)
        .map(|i| monitor.position().x + i * step)
        .filter(|&x| {
            x < position.x - WINDOW_MARGIN_PX
                || x > position.x + size.width as i32 + WINDOW_MARGIN_PX
        })
        .filter(|&x| on_taskbar(&monitor, x, y));

    let pixels = read_pixels(xs, y);
    if pixels.len() < MIN_SAMPLES {
        return Ok(None);
    }
    Ok(Some(hex(median(&pixels))))
}

/// Couleur d'accentuation choisie dans les paramètres de Windows.
fn accent_color() -> Option<String> {
    let color = UISettings::new()
        .ok()?
        .GetColorValue(UIColorType::Accent)
        .ok()?;
    Some(hex([color.R, color.G, color.B]))
}

fn system_colors(window: &WebviewWindow) -> tauri::Result<SystemColors> {
    Ok(SystemColors {
        taskbar: taskbar_color(window)?,
        accent: accent_color(),
    })
}

/// Relit les couleurs régulièrement (fond d'écran, thème, accent) et prévient le front si elles changent.
pub fn watch(window: WebviewWindow) {
    thread::spawn(move || {
        let mut last: Option<SystemColors> = None;
        loop {
            match system_colors(&window) {
                Ok(colors) if last.as_ref() != Some(&colors) => {
                    if let Err(e) = window.emit(SYSTEM_COLORS_EVENT, &colors) {
                        eprintln!("emit {SYSTEM_COLORS_EVENT} a échoué : {e}");
                    }
                    last = Some(colors);
                }
                Ok(_) => {}
                Err(e) => eprintln!("lecture des couleurs système échouée : {e}"),
            }
            thread::sleep(POLL_INTERVAL);
        }
    });
}

#[tauri::command]
pub async fn get_system_colors(window: WebviewWindow) -> Result<SystemColors, String> {
    system_colors(&window).map_err(|e| e.to_string())
}
