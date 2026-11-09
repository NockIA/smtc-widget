use std::time::{SystemTime, UNIX_EPOCH};

use crate::shared::smtc::Session;

const TICKS_PER_MS: i64 = 10_000;

const NANOS_PER_TICK: u128 = 100;

/// Écart entre l'epoch Windows (1601) et l'epoch Unix (1970), en ticks de 100 ns.
const WINDOWS_TO_UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

pub struct Timeline {
    pub position_ms: u64,
    pub duration_ms: u64,
}

impl Timeline {
    /// Lit la timeline de la session. SMTC donne la position à `LastUpdatedTime` :
    /// si le morceau est en lecture, on ajoute le temps écoulé depuis.
    pub fn estimate(session: &Session, is_playing: bool) -> windows::core::Result<Self> {
        let timeline = session.GetTimelineProperties()?;
        let start = timeline.StartTime()?.Duration;
        let end = timeline.EndTime()?.Duration;
        let duration_ms = ((end - start).max(0) / TICKS_PER_MS) as u64;

        let mut position = (timeline.Position()?.Duration - start).max(0);
        if is_playing {
            let last_updated = timeline.LastUpdatedTime()?.UniversalTime - WINDOWS_TO_UNIX_EPOCH_TICKS;
            position += (now_ticks() - last_updated).max(0);
        }

        let mut position_ms = (position / TICKS_PER_MS) as u64;
        if duration_ms > 0 {
            position_ms = position_ms.min(duration_ms);
        }

        Ok(Self {
            position_ms,
            duration_ms,
        })
    }
}

fn now_ticks() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| (d.as_nanos() / NANOS_PER_TICK) as i64)
}
