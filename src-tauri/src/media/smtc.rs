use super::models::TrackInfo;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
};

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

    Ok(Some(TrackInfo {
        title,
        artist,
        album,
    }))
}

type SessionSubscription = Option<(GlobalSystemMediaTransportControlsSession, i64)>;

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

    if let Some((old_session, token)) = guard.take() {
        let _ = old_session.RemoveMediaPropertiesChanged(token);
    }

    if let Ok(session) = manager.GetCurrentSession() {
        let app = app.clone();
        let handler = TypedEventHandler::new(move |_, _| {
            emit_current_track(&app);
            Ok(())
        });
        let token = session.MediaPropertiesChanged(&handler)?;
        *guard = Some((session, token));
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
