use super::session::{all_sessions, request_manager, resolve_session, source_id};

pub struct SourceEntry {
    pub id: String,
    pub label: String,
    pub active: bool,
}

/// "Spotify.exe" -> "Spotify" ; "SpotifyAB.SpotifyMusic_xxx!Spotify" -> "Spotify".
fn label(id: &str) -> String {
    let name = id.rsplit('!').next().unwrap_or(id);
    name.strip_suffix(".exe").unwrap_or(name).to_string()
}

/// Sources audio disponibles, sans doublon, avec celle qui est active.
pub fn list_sources(selected: Option<&str>) -> windows::core::Result<Vec<SourceEntry>> {
    let manager = request_manager()?;
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
