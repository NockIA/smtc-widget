use serde::Serialize;

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
    /// Identifiant de l'application source (ex : "Spotify.exe").
    pub source: String,
    /// Nombre de sources audio disponibles (pour afficher le bouton de swap).
    pub session_count: usize,
}
