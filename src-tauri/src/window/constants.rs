use std::time::Duration;

/// Marge à droite (en pixels logiques) sur l'écran secondaire.
pub const SECONDARY_SCREEN_RIGHT_MARGIN: f64 = 75.0;

/// Marge à droite (en pixels logiques) sur l'écran principal, pour ne pas recouvrir
/// la zone de notification.
pub const PRIMARY_SCREEN_RIGHT_MARGIN: f64 = 210.0;

/// Marge en bas (en pixels logiques) par défaut, sur tous les écrans.
pub const DEFAULT_BOTTOM_MARGIN: f64 = 0.0;

/// Intervalle entre deux vérifications du plein écran / du z-order.
pub const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Fichier (dans le dossier de config de l'appli) où sont sauvegardées les préférences.
pub const PREFERENCES_FILE: &str = "window-preferences.json";

/// Temps sans mouvement avant de sauvegarder la position (évite d'écrire pendant le drag).
pub const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

/// Écart (en pixels logiques) en dessous duquel deux positions sont considérées identiques.
pub const POSITION_EPSILON: f64 = 1.0;
