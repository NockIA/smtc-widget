use std::time::Duration;

pub const SYSTEM_COLORS_EVENT: &str = "system-colors";

/// Contrôle de secours (changement de fond d'écran, transparence de la barre).
pub const FALLBACK_INTERVAL: Duration = Duration::from_secs(60);

/// Attente après le dernier déclencheur avant de relire les couleurs.
pub const TRIGGER_DEBOUNCE: Duration = Duration::from_millis(300);

pub const MIN_SAMPLES: usize = 3;
/// Distance minimale entre un point d'échantillonnage et le widget, qui recouvre la barre.
pub const WINDOW_MARGIN_PX: i32 = 8;
/// Les points sont pris sur une ligne proche du bord de la barre, là où il y a le moins d'icônes.
pub const ROW_OFFSET_PX: i32 = 3;
