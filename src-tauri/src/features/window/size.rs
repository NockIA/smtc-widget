use std::sync::Mutex;

use tauri::PhysicalSize;

use crate::features::preferences::{DEFAULT_SCALE, DEFAULT_STRETCH};

/// Taille du widget à 100 % (pixels logiques) : celle de la fenêtre dans `tauri.conf.json`.
const BASE_WIDTH: f64 = 250.0;
const BASE_HEIGHT: f64 = 40.0;

const MIN_SCALE: f64 = 0.75;
const MAX_SCALE: f64 = 2.0;

const MIN_STRETCH: f64 = 0.75;
const MAX_STRETCH: f64 = 3.0;

/// Dimensions du widget : `scale` agrandit tout le contenu (zoom), `stretch` n'élargit que la fenêtre.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub scale: f64,
    pub stretch: f64,
}

impl Layout {
    pub fn new(scale: f64, stretch: f64) -> Self {
        Self {
            scale: clamp_scale(scale),
            stretch: clamp_stretch(stretch),
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            scale: DEFAULT_SCALE,
            stretch: DEFAULT_STRETCH,
        }
    }
}

/// Les dimensions appliquées à la fenêtre : la taille seule ne permet pas de séparer zoom et élargissement.
static CURRENT: Mutex<Layout> = Mutex::new(Layout {
    scale: DEFAULT_SCALE,
    stretch: DEFAULT_STRETCH,
});

pub fn current() -> Layout {
    *CURRENT.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn current_scale() -> f64 {
    current().scale
}

pub fn set_current(layout: Layout) {
    *CURRENT.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = layout;
}

pub fn clamp_scale(scale: f64) -> f64 {
    scale.clamp(MIN_SCALE, MAX_SCALE)
}

pub fn clamp_stretch(stretch: f64) -> f64 {
    stretch.clamp(MIN_STRETCH, MAX_STRETCH)
}

/// Taille de la fenêtre en pixels physiques, sur un écran de facteur d'échelle `dpi`.
pub fn physical_size(layout: Layout, dpi: f64) -> PhysicalSize<u32> {
    PhysicalSize::new(
        (BASE_WIDTH * layout.scale * layout.stretch * dpi).round() as u32,
        (BASE_HEIGHT * layout.scale * dpi).round() as u32,
    )
}

/// Facteur manquant d'une fenêtre de cette largeur en pixels physiques, l'autre facteur étant `other`.
pub fn factor_from_width(width: f64, other: f64, dpi: f64) -> f64 {
    width / (BASE_WIDTH * other * dpi)
}
