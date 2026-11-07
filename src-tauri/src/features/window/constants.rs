use std::time::Duration;

pub const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;

/// Évite de recouvrir la zone de notification.
pub const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

pub const DEFAULT_BOTTOM_MARGIN: f64 = 0.0;

/// Contrôle de secours du premier plan, en plus des changements de fenêtre active.
pub const TOPMOST_FALLBACK_INTERVAL_MS: u32 = 2_000;

/// Émis vers le front quand le widget est caché (`true`) ou réaffiché (`false`).
pub const VISIBILITY_EVENT: &str = "widget-visibility";

pub const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

pub const LAYOUT_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Délai pendant lequel les déplacements sont ignorés après un changement d'écrans.
pub const LAYOUT_SETTLE: Duration = Duration::from_secs(2);
