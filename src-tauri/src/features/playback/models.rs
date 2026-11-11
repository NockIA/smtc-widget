use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Position estimée au moment de la lecture (en ms).
    pub position_ms: u64,
    /// Durée du morceau (en ms), 0 si inconnue.
    pub duration_ms: u64,
    pub is_playing: bool,
    pub source: String,
}

impl TrackInfo {
    /// Même morceau dans le même état (hors position, qui avance en continu).
    pub fn is_same_state(&self, other: &Self) -> bool {
        self.title == other.title
            && self.artist == other.artist
            && self.album == other.album
            && self.source == other.source
            && self.is_playing == other.is_playing
            && self.duration_ms == other.duration_ms
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaAction {
    TogglePlayPause,
    Previous,
    Next,
}
