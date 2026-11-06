pub const MEDIA_CHANGED_EVENT: &str = "media-changed";

/// Écart entre l'epoch Windows (1601) et l'epoch Unix (1970), en ticks de 100 ns.
pub const WINDOWS_TO_UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

pub const NANOS_PER_TICK: u128 = 100;

pub const TICKS_PER_MS: i64 = 10_000;
