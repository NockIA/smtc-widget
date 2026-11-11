use std::time::Instant;

use tauri::{AppHandle, Emitter};

use super::models::TrackInfo;

const MEDIA_CHANGED_EVENT: &str = "media-changed";

/// Écart de position (en ms) au-delà duquel on considère qu'il y a eu un saut (seek).
const SEEK_THRESHOLD_MS: u64 = 1_500;

/// Dernier état émis vers le front (`track` vaut `None` pour « aucun morceau »).
struct Emitted {
    track: Option<TrackInfo>,
    at: Instant,
}

impl Emitted {
    /// Compare la position reçue à celle que le front a interpolée depuis cette émission.
    fn has_position_jumped(&self, previous: &TrackInfo, current: &TrackInfo) -> bool {
        let elapsed_ms = if previous.is_playing {
            self.at.elapsed().as_millis() as u64
        } else {
            0
        };
        current.position_ms.abs_diff(previous.position_ms + elapsed_ms) > SEEK_THRESHOLD_MS
    }
}

/// N'émet vers le front que les changements utiles : le front interpole la position entre deux.
#[derive(Default)]
pub struct TrackEmitter {
    last: Option<Emitted>,
}

impl TrackEmitter {
    fn has_changed(&self, track: Option<&TrackInfo>) -> bool {
        let Some(last) = &self.last else {
            return true;
        };
        match (last.track.as_ref(), track) {
            (None, None) => false,
            (Some(previous), Some(current)) => {
                !previous.is_same_state(current) || last.has_position_jumped(previous, current)
            }
            _ => true,
        }
    }

    pub fn emit_if_changed(&mut self, app: &AppHandle, track: Option<TrackInfo>) {
        if !self.has_changed(track.as_ref()) {
            return;
        }
        if let Err(e) = app.emit(MEDIA_CHANGED_EVENT, &track) {
            log::error!("emit {MEDIA_CHANGED_EVENT} a échoué : {e}");
            return;
        }
        self.last = Some(Emitted {
            track,
            at: Instant::now(),
        });
    }
}
