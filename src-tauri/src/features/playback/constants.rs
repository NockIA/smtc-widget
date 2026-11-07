use std::time::Duration;

pub const MEDIA_CHANGED_EVENT: &str = "media-changed";

/// Attente après le dernier événement SMTC avant de relire le morceau (regroupe les rafales).
pub const EVENT_DEBOUNCE: Duration = Duration::from_millis(100);

/// Écart de position (en ms) au-delà duquel on considère qu'il y a eu un saut (seek).
pub const SEEK_THRESHOLD_MS: i64 = 1_500;

/// Écart entre l'epoch Windows (1601) et l'epoch Unix (1970), en ticks de 100 ns.
pub const WINDOWS_TO_UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

pub const NANOS_PER_TICK: u128 = 100;

pub const TICKS_PER_MS: i64 = 10_000;
