use std::time::Instant;

use tauri::{AppHandle, Emitter};

use super::constants::{MEDIA_CHANGED_EVENT, SEEK_THRESHOLD_MS};
use super::models::TrackInfo;

/// N'émet vers le front que les changements utiles : le front interpole la position entre deux.
#[derive(Default)]
pub struct TrackEmitter {
    /// `None` = rien émis encore ; `Some(None)` = « aucun morceau » émis.
    last: Option<Option<TrackInfo>>,
    emitted_at: Option<Instant>,
}

impl TrackEmitter {
    fn has_changed(&self, track: &Option<TrackInfo>) -> bool {
        let (Some(last), Some(emitted_at)) = (&self.last, self.emitted_at) else {
            return true;
        };
        match (last, track) {
            (None, None) => false,
            (Some(last), Some(track)) => {
                if !last.same_state(track) {
                    return true;
                }
                let elapsed = if last.is_playing {
                    emitted_at.elapsed().as_millis() as i64
                } else {
                    0
                };
                let expected = last.position_ms as i64 + elapsed;
                (track.position_ms as i64 - expected).abs() > SEEK_THRESHOLD_MS
            }
            _ => true,
        }
    }

    pub fn emit_if_changed(&mut self, app: &AppHandle, track: Option<TrackInfo>) {
        if !self.has_changed(&track) {
            return;
        }
        if let Err(e) = app.emit(MEDIA_CHANGED_EVENT, &track) {
            eprintln!("emit {MEDIA_CHANGED_EVENT} a échoué : {e}");
            return;
        }
        self.last = Some(track);
        self.emitted_at = Some(Instant::now());
    }
}
