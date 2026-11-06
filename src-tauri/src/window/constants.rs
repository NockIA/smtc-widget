use std::time::Duration;

pub const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;

/// Évite de recouvrir la zone de notification.
pub const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

pub const DEFAULT_BOTTOM_MARGIN: f64 = 0.0;

pub const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub const PREFERENCES_FILE: &str = "window-preferences.json";

pub const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

/// Écart (en pixels logiques) en dessous duquel deux positions sont considérées identiques.
pub const POSITION_EPSILON: f64 = 1.0;
