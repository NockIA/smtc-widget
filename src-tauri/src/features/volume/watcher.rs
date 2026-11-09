// Lints levés par le code que génère `#[implement]`, hors de notre contrôle.
#![allow(clippy::inline_always, clippy::ref_as_ptr)]

use std::iter;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    AudioSessionDisconnectReason, AudioSessionState, AudioSessionStateExpired, DEVICE_STATE, EDataFlow, ERole,
    IAudioSessionControl, IAudioSessionEvents, IAudioSessionEvents_Impl, IAudioSessionManager2,
    IAudioSessionNotification, IAudioSessionNotification_Impl, IMMDeviceEnumerator, IMMNotificationClient,
    IMMNotificationClient_Impl, eMultimedia, eRender,
};
use windows::core::{BOOL, GUID, PCWSTR, Ref, implement};

use super::service::{self, ComGuard, WIDGET_CONTEXT};

/// Nom de l'événement écouté par le front.
const VOLUME_CHANGED_EVENT: &str = "volume-changed";

/// Regroupe les rafales (glissé du curseur dans le mélangeur Windows).
const EVENT_DEBOUNCE: Duration = Duration::from_millis(50);

enum Message {
    /// La source suivie par le lecteur a changé.
    Follow(Option<String>),
    /// Les sessions audio ou le périphérique de sortie ont changé : se réabonner.
    Resubscribe,
    VolumeChanged,
}

static SENDER: OnceLock<Sender<Message>> = OnceLock::new();

/// Indique la source SMTC dont le volume doit être suivi.
pub fn follow_source(source: Option<String>) {
    if let Some(tx) = SENDER.get() {
        let _ = tx.send(Message::Follow(source));
    }
}

#[implement(IAudioSessionEvents)]
struct SessionEvents {
    tx: Sender<Message>,
}

impl IAudioSessionEvents_Impl for SessionEvents_Impl {
    fn OnSimpleVolumeChanged(&self, _volume: f32, _mute: BOOL, context: *const GUID) -> windows::core::Result<()> {
        // Changement fait par le widget : le front connaît déjà la valeur, la renvoyer ferait sauter le curseur.
        if unsafe { context.as_ref() } != Some(&WIDGET_CONTEXT) {
            let _ = self.tx.send(Message::VolumeChanged);
        }
        Ok(())
    }

    fn OnStateChanged(&self, state: AudioSessionState) -> windows::core::Result<()> {
        if state == AudioSessionStateExpired {
            let _ = self.tx.send(Message::Resubscribe);
        }
        Ok(())
    }

    fn OnSessionDisconnected(&self, _reason: AudioSessionDisconnectReason) -> windows::core::Result<()> {
        let _ = self.tx.send(Message::Resubscribe);
        Ok(())
    }

    fn OnDisplayNameChanged(&self, _name: &PCWSTR, _context: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnIconPathChanged(&self, _path: &PCWSTR, _context: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnChannelVolumeChanged(
        &self,
        _count: u32,
        _volumes: *const f32,
        _changed: u32,
        _context: *const GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnGroupingParamChanged(&self, _param: *const GUID, _context: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }
}

/// Une application peut ouvrir sa session audio après le début de la lecture.
#[implement(IAudioSessionNotification)]
struct SessionCreated {
    tx: Sender<Message>,
}

impl IAudioSessionNotification_Impl for SessionCreated_Impl {
    fn OnSessionCreated(&self, _session: Ref<IAudioSessionControl>) -> windows::core::Result<()> {
        let _ = self.tx.send(Message::Resubscribe);
        Ok(())
    }
}

/// Changement de périphérique de sortie (casque, haut-parleurs) : les sessions suivies ne sont plus les bonnes.
#[implement(IMMNotificationClient)]
struct DeviceEvents {
    tx: Sender<Message>,
}

impl IMMNotificationClient_Impl for DeviceEvents_Impl {
    fn OnDefaultDeviceChanged(&self, flow: EDataFlow, role: ERole, _device: &PCWSTR) -> windows::core::Result<()> {
        if flow == eRender && role == eMultimedia {
            let _ = self.tx.send(Message::Resubscribe);
        }
        Ok(())
    }

    fn OnDeviceStateChanged(&self, _device: &PCWSTR, _state: DEVICE_STATE) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnDeviceAdded(&self, _device: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _device: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _device: &PCWSTR, _key: &PROPERTYKEY) -> windows::core::Result<()> {
        Ok(())
    }
}

/// Abonnements aux sessions audio de la source suivie ; tout est désabonné à la destruction.
struct Subscription {
    manager: IAudioSessionManager2,
    created: IAudioSessionNotification,
    sessions: Vec<IAudioSessionControl>,
    events: Vec<IAudioSessionEvents>,
}

impl Subscription {
    fn new(enumerator: &IMMDeviceEnumerator, source: &str, tx: &Sender<Message>) -> windows::core::Result<Self> {
        let mut subscription = Self {
            manager: service::session_manager(enumerator)?,
            created: SessionCreated { tx: tx.clone() }.into(),
            sessions: Vec::new(),
            events: Vec::new(),
        };
        // `source_sessions` énumère les sessions, ce que Windows exige avant de notifier les créations.
        for session in service::source_sessions(&subscription.manager, source)? {
            let events: IAudioSessionEvents = SessionEvents { tx: tx.clone() }.into();
            unsafe { session.RegisterAudioSessionNotification(&events)? };
            subscription.sessions.push(session);
            subscription.events.push(events);
        }
        unsafe {
            subscription
                .manager
                .RegisterSessionNotification(&subscription.created)?;
        };
        Ok(subscription)
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        unsafe {
            let _ = self.manager.UnregisterSessionNotification(&self.created);
            for (session, events) in self.sessions.iter().zip(&self.events) {
                let _ = session.UnregisterAudioSessionNotification(events);
            }
        }
    }
}

fn subscribe(enumerator: &IMMDeviceEnumerator, source: Option<&str>, tx: &Sender<Message>) -> Option<Subscription> {
    Subscription::new(enumerator, source?, tx)
        .inspect_err(|e| log::error!("abonnement au volume a échoué : {e}"))
        .ok()
}

fn current_volume(subscription: Option<&Subscription>) -> Option<f32> {
    service::max_volume(&subscription?.sessions)
        .inspect_err(|e| log::error!("lecture du volume a échoué : {e}"))
        .ok()
        .flatten()
}

fn run(app: AppHandle, rx: Receiver<Message>, tx: Sender<Message>) -> windows::core::Result<()> {
    let _com = ComGuard::new();
    let enumerator = service::device_enumerator()?;
    let device_events: IMMNotificationClient = DeviceEvents { tx: tx.clone() }.into();
    unsafe { enumerator.RegisterEndpointNotificationCallback(&device_events)? };

    let mut source: Option<String> = None;
    let mut subscription: Option<Subscription> = None;

    while let Ok(first) = rx.recv() {
        let mut needs_resubscribe = false;
        let mut changed = false;
        for message in iter::once(first).chain(iter::from_fn(|| rx.recv_timeout(EVENT_DEBOUNCE).ok())) {
            match message {
                Message::Follow(followed) if followed != source => {
                    source = followed;
                    needs_resubscribe = true;
                }
                Message::Follow(_) => {}
                Message::Resubscribe => needs_resubscribe = true,
                Message::VolumeChanged => changed = true,
            }
        }

        if needs_resubscribe {
            drop(subscription.take());
            subscription = subscribe(&enumerator, source.as_deref(), &tx);
        }
        if needs_resubscribe || changed {
            let volume = current_volume(subscription.as_ref());
            if let Err(e) = app.emit(VOLUME_CHANGED_EVENT, volume) {
                log::error!("emit {VOLUME_CHANGED_EVENT} a échoué : {e}");
            }
        }
    }
    Ok(())
}

/// Pousse au front le volume de la source suivie (`follow_source`) à chaque changement,
/// y compris depuis le mélangeur Windows.
pub fn start_watching(app: AppHandle) {
    let (tx, rx) = mpsc::channel();
    if SENDER.set(tx.clone()).is_err() {
        return;
    }
    thread::spawn(move || {
        if let Err(e) = run(app, rx, tx) {
            log::error!("surveillance du volume a échoué : {e}");
        }
    });
}
