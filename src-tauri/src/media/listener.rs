use std::sync::mpsc::{self, Receiver, Sender};

use tauri::{AppHandle, Emitter, Manager};
use windows::Foundation::TypedEventHandler;

use super::constants::MEDIA_CHANGED_EVENT;
use super::session::{Session, SmtcManager, next_source_id, request_manager, resolve_session};
use super::state::{MediaCommand, MediaState};
use super::track::get_current_track;

/// Crée un handler d'événement qui relit le morceau et l'envoie au front.
macro_rules! emit_on_event {
    ($app:expr) => {{
        let app = $app.clone();
        TypedEventHandler::new(move |_, _| {
            emit_current_track(&app);
            Ok(())
        })
    }};
}

/// Crée un handler d'événement qui demande au thread d'écoute de se resynchroniser.
macro_rules! refresh_on_event {
    ($tx:expr) => {{
        let tx = $tx.clone();
        TypedEventHandler::new(move |_, _| {
            let _ = tx.send(MediaCommand::Refresh);
            Ok(())
        })
    }};
}

/// Abonnement aux événements d'une session (propriétés, timeline, lecture).
struct SessionSubscription {
    session: Session,
    media_token: i64,
    timeline_token: i64,
    playback_token: i64,
}

impl SessionSubscription {
    fn new(session: Session, app: &AppHandle) -> windows::core::Result<Self> {
        Ok(Self {
            media_token: session.MediaPropertiesChanged(&emit_on_event!(app))?,
            timeline_token: session.TimelinePropertiesChanged(&emit_on_event!(app))?,
            playback_token: session.PlaybackInfoChanged(&emit_on_event!(app))?,
            session,
        })
    }

    fn cancel(self) {
        let _ = self.session.RemoveMediaPropertiesChanged(self.media_token);
        let _ = self
            .session
            .RemoveTimelinePropertiesChanged(self.timeline_token);
        let _ = self.session.RemovePlaybackInfoChanged(self.playback_token);
    }
}

/// Relit le morceau courant et l'envoie au front.
fn emit_current_track(app: &AppHandle) {
    let selected = app.state::<MediaState>().selected();
    match get_current_track(selected.as_deref()) {
        Ok(track) => {
            if let Err(e) = app.emit(MEDIA_CHANGED_EVENT, track) {
                eprintln!("emit {MEDIA_CHANGED_EVENT} a échoué : {e}");
            }
        }
        Err(e) => eprintln!("get_current_track a échoué : {e}"),
    }
}

/// Retire l'ancien abonnement et s'abonne à la session résolue.
fn resubscribe(
    manager: &SmtcManager,
    app: &AppHandle,
    subscription: &mut Option<SessionSubscription>,
) -> windows::core::Result<()> {
    if let Some(old) = subscription.take() {
        old.cancel();
    }

    let selected = app.state::<MediaState>().selected();
    if let Some(session) = resolve_session(manager, selected.as_deref()) {
        *subscription = Some(SessionSubscription::new(session, app)?);
    }
    Ok(())
}

/// Mémorise la source suivante comme source choisie.
fn select_next_session(manager: &SmtcManager, app: &AppHandle) {
    let state = app.state::<MediaState>();
    if let Some(id) = next_source_id(manager, state.selected().as_deref()) {
        state.set_selected(id);
    }
}

fn run_listener(
    app: AppHandle,
    rx: Receiver<MediaCommand>,
    tx: Sender<MediaCommand>,
) -> windows::core::Result<()> {
    let manager = request_manager()?;
    let mut subscription = None;

    // Session active de Windows qui change, ou source qui apparaît / disparaît.
    manager.CurrentSessionChanged(&refresh_on_event!(tx))?;
    manager.SessionsChanged(&refresh_on_event!(tx))?;

    resubscribe(&manager, &app, &mut subscription)?;
    emit_current_track(&app);

    // Le thread (donc manager et session) reste en vie tant que l'appli tourne.
    while let Ok(command) = rx.recv() {
        if let MediaCommand::NextSession = command {
            select_next_session(&manager, &app);
        }
        if let Err(e) = resubscribe(&manager, &app, &mut subscription) {
            eprintln!("réabonnement à la session a échoué : {e}");
        }
        emit_current_track(&app);
    }
    Ok(())
}

pub fn start_listening(app: AppHandle) {
    let (tx, rx) = mpsc::channel();
    // Géré avant le thread pour que les commandes puissent déjà l'utiliser.
    app.manage(MediaState::new(tx.clone()));

    std::thread::spawn(move || {
        if let Err(e) = run_listener(app, rx, tx) {
            eprintln!("start_listening a échoué : {e}");
        }
    });
}
