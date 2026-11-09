use std::sync::mpsc::{self, Receiver, Sender};

use tauri::{AppHandle, Manager};
use windows::Foundation::TypedEventHandler;

use super::constants::EVENT_DEBOUNCE;
use super::emitter::TrackEmitter;
use super::services::get_current_track;
use crate::features::audio_source::AudioSourceState;
use crate::shared::smtc::{Session, SmtcManager, resolve_session, smtc_manager};

enum ListenerEvent {
    /// La source ou la liste des sessions a changé : se réabonner.
    Resubscribe,
    /// Le morceau ou son état a changé dans la session suivie.
    TrackChanged,
}

macro_rules! send_on_event {
    ($tx:expr, $event:expr) => {{
        let tx = $tx.clone();
        TypedEventHandler::new(move |_, _| {
            let _ = tx.send($event);
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
    fn new(session: Session, tx: &Sender<ListenerEvent>) -> windows::core::Result<Self> {
        Ok(Self {
            media_token: session.MediaPropertiesChanged(&send_on_event!(tx, ListenerEvent::TrackChanged))?,
            timeline_token: session.TimelinePropertiesChanged(&send_on_event!(tx, ListenerEvent::TrackChanged))?,
            playback_token: session.PlaybackInfoChanged(&send_on_event!(tx, ListenerEvent::TrackChanged))?,
            session,
        })
    }

    fn cancel(self) {
        let _ = self.session.RemoveMediaPropertiesChanged(self.media_token);
        let _ = self.session.RemoveTimelinePropertiesChanged(self.timeline_token);
        let _ = self.session.RemovePlaybackInfoChanged(self.playback_token);
    }
}

fn resubscribe(
    manager: &SmtcManager,
    app: &AppHandle,
    tx: &Sender<ListenerEvent>,
    subscription: &mut Option<SessionSubscription>,
) -> windows::core::Result<()> {
    if let Some(old) = subscription.take() {
        old.cancel();
    }

    let selected = app.state::<AudioSourceState>().selected();
    if let Some(session) = resolve_session(manager, selected.as_deref()) {
        *subscription = Some(SessionSubscription::new(session, tx)?);
    }
    Ok(())
}

fn publish(app: &AppHandle, emitter: &mut TrackEmitter) {
    let selected = app.state::<AudioSourceState>().selected();
    match get_current_track(selected.as_deref()) {
        Ok(track) => emitter.emit_if_changed(app, track),
        Err(e) => log::error!("get_current_track a échoué : {e}"),
    }
}

fn run_listener(app: AppHandle, rx: Receiver<ListenerEvent>, tx: Sender<ListenerEvent>) -> windows::core::Result<()> {
    let manager = smtc_manager()?;
    let mut subscription = None;
    let mut emitter = TrackEmitter::default();

    manager.CurrentSessionChanged(&send_on_event!(tx, ListenerEvent::Resubscribe))?;
    manager.SessionsChanged(&send_on_event!(tx, ListenerEvent::Resubscribe))?;

    resubscribe(&manager, &app, &tx, &mut subscription)?;
    publish(&app, &mut emitter);

    // Le thread garde manager et session en vie.
    while let Ok(event) = rx.recv() {
        let mut needs_resubscribe = matches!(event, ListenerEvent::Resubscribe);
        while let Ok(next) = rx.recv_timeout(EVENT_DEBOUNCE) {
            needs_resubscribe |= matches!(next, ListenerEvent::Resubscribe);
        }

        if needs_resubscribe && let Err(e) = resubscribe(&manager, &app, &tx, &mut subscription) {
            log::error!("réabonnement à la session a échoué : {e}");
        }
        publish(&app, &mut emitter);
    }
    Ok(())
}

pub fn start_listening(app: AppHandle) {
    let (tx, rx) = mpsc::channel();

    // Géré avant le thread pour que les commandes puissent déjà l'utiliser.
    let on_source_change = tx.clone();
    app.manage(AudioSourceState::new(move || {
        let _ = on_source_change.send(ListenerEvent::Resubscribe);
    }));

    std::thread::spawn(move || {
        if let Err(e) = run_listener(app, rx, tx) {
            log::error!("start_listening a échoué : {e}");
        }
    });
}
