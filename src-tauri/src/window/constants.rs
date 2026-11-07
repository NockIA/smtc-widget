use std::time::Duration;

pub const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;

/// Évite de recouvrir la zone de notification.
pub const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

pub const DEFAULT_BOTTOM_MARGIN: f64 = 0.0;

pub const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

pub const LAYOUT_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Délai pendant lequel les déplacements sont ignorés après un changement d'écrans.
pub const LAYOUT_SETTLE: Duration = Duration::from_secs(2);
