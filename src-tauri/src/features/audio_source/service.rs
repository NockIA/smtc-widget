use super::models::SourceEntry;
use crate::shared::smtc::{all_sessions, resolve_session, smtc_manager, source_id};

/// "Spotify.exe" -> "Spotify" ; "SpotifyAB.SpotifyMusic_xxx!Spotify" -> "Spotify".
fn label(id: &str) -> String {
    let name = id.rsplit('!').next().unwrap_or(id);
    name.strip_suffix(".exe").unwrap_or(name).to_string()
}

/// Sources audio disponibles, sans doublon, avec celle qui est active.
pub fn list_sources(selected: Option<&str>) -> windows::core::Result<Vec<SourceEntry>> {
    let manager = smtc_manager()?;
    let active = resolve_session(&manager, selected).map(|s| source_id(&s));

    let mut sources: Vec<SourceEntry> = Vec::new();
    for session in all_sessions(&manager) {
        let id = source_id(&session);
        if sources.iter().any(|s| s.id == id) {
            continue;
        }
        sources.push(SourceEntry {
            label: label(&id),
            active: active.as_deref() == Some(id.as_str()),
            id,
        });
    }
    Ok(sources)
}
