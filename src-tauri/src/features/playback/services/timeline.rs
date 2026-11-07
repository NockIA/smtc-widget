use std::time::{SystemTime, UNIX_EPOCH};

use crate::features::playback::constants::{
    NANOS_PER_TICK, TICKS_PER_MS, WINDOWS_TO_UNIX_EPOCH_TICKS,
};
use crate::shared::smtc::Session;

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
            let last_updated =
                timeline.LastUpdatedTime()?.UniversalTime - WINDOWS_TO_UNIX_EPOCH_TICKS;
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
        .map(|d| (d.as_nanos() / NANOS_PER_TICK) as i64)
        .unwrap_or(0)
}
