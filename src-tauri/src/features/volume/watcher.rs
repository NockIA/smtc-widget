// Lints levés par le code que génère `#[implement]`, hors de notre contrôle.
#![allow(clippy::inline_always, clippy::ref_as_ptr)]

use std::iter;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
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

const VOLUME_CHANGED_EVENT: &str = "volume-changed";

/// Regroupe les rafales (glissé du curseur dans le mélangeur Windows).
const EVENT_DEBOUNCE: Duration = Duration::from_millis(50);

enum Message {
    /// La source suivie par le lecteur a changé.
    Follow(Option<String>),
    /// Volume demandé par le widget.
    SetVolume(f32),
    /// Les sessions audio ou le périphérique de sortie ont changé : se réabonner.
    Resubscribe,
    VolumeChanged,
}

/// Volume de la source suivie : relu à chaque changement et poussé au front, y compris depuis le mélangeur Windows.
/// Un thread dédié garde les sessions audio ouvertes, ce qui évite de les rechercher à chaque réglage du curseur.
pub struct VolumeWatcher {
    tx: Sender<Message>,
    /// Dernier volume connu, lu par le front au démarrage (l'événement peut partir avant qu'il écoute).
    current: Arc<Mutex<Option<f32>>>,
}

impl VolumeWatcher {
    pub fn start(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::channel();
        let current = Arc::new(Mutex::new(None));

        let worker_tx = tx.clone();
        let worker_current = current.clone();
        // Les objets COM sont créés sur le thread qui les utilise.
        thread::spawn(move || {
            if let Err(e) = Worker::new(app, worker_tx, worker_current).map(|worker| worker.run(&rx)) {
                log::error!("surveillance du volume a échoué : {e}");
            }
        });
        Self { tx, current }
    }

    /// Indique la source SMTC dont le volume doit être suivi.
    pub fn follow_source(&self, source: Option<String>) {
        let _ = self.tx.send(Message::Follow(source));
    }

    pub fn volume(&self) -> Option<f32> {
        *self.current.lock().unwrap()
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.tx.send(Message::SetVolume(volume));
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

/// État du thread de surveillance.
struct Worker {
    app: AppHandle,
    tx: Sender<Message>,
    current: Arc<Mutex<Option<f32>>>,
    enumerator: IMMDeviceEnumerator,
    /// Windows ne retient pas cet objet : il doit rester vivant tant qu'il est enregistré.
    device_events: IMMNotificationClient,
    source: Option<String>,
    subscription: Option<Subscription>,
    /// Garde COM du thread : à libérer en dernier.
    _com: ComGuard,
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = unsafe {
            self.enumerator
                .UnregisterEndpointNotificationCallback(&self.device_events)
        };
    }
}

impl Worker {
    fn new(app: AppHandle, tx: Sender<Message>, current: Arc<Mutex<Option<f32>>>) -> windows::core::Result<Self> {
        let com = ComGuard::new();
        let enumerator = service::device_enumerator()?;
        let device_events: IMMNotificationClient = DeviceEvents { tx: tx.clone() }.into();
        unsafe { enumerator.RegisterEndpointNotificationCallback(&device_events)? };
        Ok(Self {
            app,
            tx,
            current,
            enumerator,
            device_events,
            source: None,
            subscription: None,
            _com: com,
        })
    }

    fn run(mut self, rx: &Receiver<Message>) {
        while let Ok(first) = rx.recv() {
            let mut needs_resubscribe = false;
            let mut changed = false;
            for message in iter::once(first).chain(iter::from_fn(|| rx.recv_timeout(EVENT_DEBOUNCE).ok())) {
                match message {
                    // Appliqué tout de suite : un glissé du curseur envoie des messages en continu et ne laisserait
                    // jamais la fin du regroupement arriver.
                    Message::SetVolume(volume) => self.set_volume(volume),
                    Message::Follow(followed) if followed != self.source => {
                        self.source = followed;
                        needs_resubscribe = true;
                    }
                    Message::Follow(_) => {}
                    Message::Resubscribe => needs_resubscribe = true,
                    Message::VolumeChanged => changed = true,
                }
            }

            if needs_resubscribe {
                self.resubscribe();
            }
            if needs_resubscribe || changed {
                self.publish();
            }
        }
    }

    fn resubscribe(&mut self) {
        drop(self.subscription.take());
        let Some(source) = self.source.as_deref() else {
            return;
        };
        self.subscription = Subscription::new(&self.enumerator, source, &self.tx)
            .inspect_err(|e| log::error!("abonnement au volume a échoué : {e}"))
            .ok();
    }

    fn read_volume(&self) -> Option<f32> {
        service::max_volume(&self.subscription.as_ref()?.sessions)
            .inspect_err(|e| log::error!("lecture du volume a échoué : {e}"))
            .ok()
            .flatten()
    }

    /// Mémorise le volume et le pousse au front.
    fn publish(&self) {
        let volume = self.read_volume();
        *self.current.lock().unwrap() = volume;
        if let Err(e) = self.app.emit(VOLUME_CHANGED_EVENT, volume) {
            log::error!("emit {VOLUME_CHANGED_EVENT} a échoué : {e}");
        }
    }

    /// Le front connaît déjà la valeur qu'il demande : on met seulement à jour la mémoire, sans événement.
    fn set_volume(&self, volume: f32) {
        let Some(subscription) = &self.subscription else {
            return;
        };
        match service::set_volume(&subscription.sessions, volume) {
            Ok(()) => *self.current.lock().unwrap() = self.read_volume(),
            Err(e) => log::error!("réglage du volume a échoué : {e}"),
        }
    }
}
