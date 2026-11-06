use std::time::Duration;

/// Marge à droite (en pixels logiques) sur l'écran secondaire.
pub const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;

/// Marge à droite (en pixels logiques) sur l'écran principal, pour ne pas recouvrir
/// la zone de notification.
pub const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

/// Intervalle entre deux vérifications du plein écran / du z-order.
pub const POLL_INTERVAL: Duration = Duration::from_millis(100);
