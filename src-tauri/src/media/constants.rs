/// Nom de l'événement Tauri émis vers le front quand le morceau (ou son état) change.
pub const MEDIA_CHANGED_EVENT: &str = "media-changed";

/// Écart entre l'epoch Windows (1601) et l'epoch Unix (1970), en ticks de 100 ns.
pub const WINDOWS_TO_UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

/// Nombre de nanosecondes dans un tick Windows.
pub const NANOS_PER_TICK: u128 = 100;

/// Nombre de ticks Windows dans une milliseconde.
pub const TICKS_PER_MS: i64 = 10_000;
