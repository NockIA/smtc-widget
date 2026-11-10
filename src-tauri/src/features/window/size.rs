use tauri::PhysicalSize;

/// Taille du widget à 100 % (pixels logiques) : celle de la fenêtre dans `tauri.conf.json`.
const BASE_WIDTH: f64 = 250.0;
const BASE_HEIGHT: f64 = 40.0;

const MIN_SCALE: f64 = 0.75;
const MAX_SCALE: f64 = 2.0;

pub fn clamp_scale(scale: f64) -> f64 {
    scale.clamp(MIN_SCALE, MAX_SCALE)
}

/// Taille de la fenêtre en pixels physiques pour cette échelle, sur un écran de facteur d'échelle `dpi`.
pub fn physical_size(scale: f64, dpi: f64) -> PhysicalSize<u32> {
    PhysicalSize::new(
        (BASE_WIDTH * scale * dpi).round() as u32,
        (BASE_HEIGHT * scale * dpi).round() as u32,
    )
}

/// Échelle (1.0 = taille de base) d'une fenêtre de cette largeur en pixels physiques.
pub fn scale_from_width(width: f64, dpi: f64) -> f64 {
    width / (BASE_WIDTH * dpi)
}
