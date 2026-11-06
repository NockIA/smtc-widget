use std::sync::mpsc::{self, Receiver, Sender};

use tauri::{AppHandle, Emitter, Manager};
use windows::Foundation::TypedEventHandler;

use super::constants::MEDIA_CHANGED_EVENT;
use super::session::{Session, SmtcManager, request_manager, resolve_session};
use super::state::MediaState;
use super::track::get_current_track;

macro_rules! emit_on_event {
    ($app:expr) => {{
        let app = $app.clone();
        TypedEventHandler::new(move |_, _| {
            emit_current_track(&app);
            Ok(())
        })
    }};
}

macro_rules! refresh_on_event {
    ($tx:expr) => {{
        let tx = $tx.clone();
        TypedEventHandler::new(move |_, _| {
            let _ = tx.send(());
            Ok(())
        })
    }};
}

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

fn run_listener(
    app: AppHandle,
    rx: Receiver<()>,
    tx: Sender<()>,
) -> windows::core::Result<()> {
    let manager = request_manager()?;
    let mut subscription = None;

    manager.CurrentSessionChanged(&refresh_on_event!(tx))?;
    manager.SessionsChanged(&refresh_on_event!(tx))?;

    resubscribe(&manager, &app, &mut subscription)?;
    emit_current_track(&app);

    // Le thread garde manager et session en vie.
    while rx.recv().is_ok() {
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
