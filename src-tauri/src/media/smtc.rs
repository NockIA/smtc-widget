use super::models::TrackInfo;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

/// Écart entre l'epoch Windows (1601) et l'epoch Unix (1970), en unités de 100 ns.
const WINDOWS_TO_UNIX_EPOCH_100NS: i64 = 116_444_736_000_000_000;

/// Position courante estimée on ajoute le temps écoulé depuis si le morceau est en lecture.
fn estimate_position_ms(
    session: &GlobalSystemMediaTransportControlsSession,
    is_playing: bool,
) -> windows::core::Result<(u64, u64)> {
    let timeline = session.GetTimelineProperties()?;
    let start = timeline.StartTime()?.Duration;
    let end = timeline.EndTime()?.Duration;
    let duration_ms = ((end - start).max(0) / 10_000) as u64;

    let mut position = (timeline.Position()?.Duration - start).max(0);

    if is_playing {
        let last_updated = timeline.LastUpdatedTime()?.UniversalTime - WINDOWS_TO_UNIX_EPOCH_100NS;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_nanos() / 100) as i64)
            .unwrap_or(0);
        position += (now - last_updated).max(0);
    }

    let mut position_ms = (position / 10_000) as u64;
    if duration_ms > 0 {
        position_ms = position_ms.min(duration_ms);
    }
    Ok((position_ms, duration_ms))
}

pub fn get_current_track() -> windows::core::Result<Option<TrackInfo>> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()?;

    let session = match manager.GetCurrentSession() {
        Ok(session) => session,
        Err(_) => return Ok(None),
    };

    let props = session.TryGetMediaPropertiesAsync()?.join()?;

    let title = props.Title()?.to_string();
    let artist = props.Artist()?.to_string();
    let album = props.AlbumTitle()?.to_string();

    let is_playing = session.GetPlaybackInfo()?.PlaybackStatus()?
        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
    let (position_ms, duration_ms) = estimate_position_ms(&session, is_playing)?;

    Ok(Some(TrackInfo {
        title,
        artist,
        album,
        position_ms,
        duration_ms,
        is_playing,
    }))
}

/// Session abonnée + tokens (media properties, timeline, playback info).
type SessionSubscription = Option<(GlobalSystemMediaTransportControlsSession, [i64; 3])>;

/// Relit le morceau courant et l'envoie au front.
fn emit_current_track(app: &AppHandle) {
    match get_current_track() {
        Ok(track) => {
            if let Err(e) = app.emit("media-changed", track) {
                eprintln!("emit media-changed a échoué : {e}");
            }
        }
        Err(e) => eprintln!("get_current_track a échoué : {e}"),
    }
}

/// Branche MediaPropertiesChanged sur la session courante,
/// en retirant l'abonnement de l'ancienne session.
fn subscribe_to_current_session(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    app: &AppHandle,
    slot: &Arc<Mutex<SessionSubscription>>,
) -> windows::core::Result<()> {
    let mut guard = slot.lock().unwrap();

    if let Some((old_session, tokens)) = guard.take() {
        let _ = old_session.RemoveMediaPropertiesChanged(tokens[0]);
        let _ = old_session.RemoveTimelinePropertiesChanged(tokens[1]);
        let _ = old_session.RemovePlaybackInfoChanged(tokens[2]);
    }

    if let Ok(session) = manager.GetCurrentSession() {
        let on_media = {
            let app = app.clone();
            TypedEventHandler::new(move |_, _| {
                emit_current_track(&app);
                Ok(())
            })
        };
        let on_timeline = {
            let app = app.clone();
            TypedEventHandler::new(move |_, _| {
                emit_current_track(&app);
                Ok(())
            })
        };
        let on_playback = {
            let app = app.clone();
            TypedEventHandler::new(move |_, _| {
                emit_current_track(&app);
                Ok(())
            })
        };
        let tokens = [
            session.MediaPropertiesChanged(&on_media)?,
            session.TimelinePropertiesChanged(&on_timeline)?,
            session.PlaybackInfoChanged(&on_playback)?,
        ];
        *guard = Some((session, tokens));
    }

    Ok(())
}

fn run_listener(app: AppHandle) -> windows::core::Result<()> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()?;
    let slot: Arc<Mutex<SessionSubscription>> = Arc::new(Mutex::new(None));

    // Changement de morceau dans la session actuelle.
    subscribe_to_current_session(&manager, &app, &slot)?;

    // Changement de session active (ex : Spotify -> navigateur).
    let handler = {
        let manager = manager.clone();
        let app = app.clone();
        let slot = slot.clone();
        TypedEventHandler::new(move |_, _| {
            if let Err(e) = subscribe_to_current_session(&manager, &app, &slot) {
                eprintln!("réabonnement à la session a échoué : {e}");
            }
            emit_current_track(&app);
            Ok(())
        })
    };
    manager.CurrentSessionChanged(&handler)?;

    // Garde le thread en vie.
    loop {
        std::thread::park();
    }
}

pub fn start_listening(app: AppHandle) {
    std::thread::spawn(move || {
        if let Err(e) = run_listener(app) {
            eprintln!("start_listening a échoué : {e}");
        }
    });
}
