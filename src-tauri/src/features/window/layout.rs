use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{Manager, WebviewWindow};

use super::foreground::is_hidden;
use super::placement::{compute_monitors_signature, place_on_preferred_monitor};
use crate::features::preferences::PreferencesStore;

/// Délai pendant lequel les déplacements sont ignorés après un changement d'écrans.
const LAYOUT_SETTLE: Duration = Duration::from_secs(2);

const LAYOUT_POLL_INTERVAL: Duration = Duration::from_secs(5);

static KNOWN_SIGNATURE: Mutex<String> = Mutex::new(String::new());
static SETTLING_UNTIL: Mutex<Option<Instant>> = Mutex::new(None);

/// Faux pendant qu'un écran vient d'être branché ou débranché : Windows déplace alors
/// la fenêtre lui-même, et ce déplacement ne doit pas être sauvegardé comme un choix.
pub fn is_stable(window: &WebviewWindow) -> tauri::Result<bool> {
    let settled = SETTLING_UNTIL
        .lock()
        .unwrap()
        .is_none_or(|until| Instant::now() >= until);
    Ok(settled && compute_monitors_signature(window)? == *KNOWN_SIGNATURE.lock().unwrap())
}

/// Replace la fenêtre quand la liste des écrans change.
pub fn watch(window: WebviewWindow) {
    match compute_monitors_signature(&window) {
        Ok(signature) => *KNOWN_SIGNATURE.lock().unwrap() = signature,
        Err(e) => {
            log::error!("lecture des écrans a échoué : {e}");
            return;
        }
    }

    thread::spawn(move || {
        loop {
            thread::sleep(LAYOUT_POLL_INTERVAL);
            if is_hidden() {
                continue;
            }

            let Ok(signature) = compute_monitors_signature(&window) else {
                continue;
            };
            if signature == *KNOWN_SIGNATURE.lock().unwrap() {
                continue;
            }

            *SETTLING_UNTIL.lock().unwrap() = Some(Instant::now() + LAYOUT_SETTLE);
            *KNOWN_SIGNATURE.lock().unwrap() = signature;

            let prefs = window.state::<PreferencesStore>().snapshot();
            if let Err(e) = place_on_preferred_monitor(&window, &prefs) {
                log::error!("replacement de la fenêtre a échoué : {e}");
            }
        }
    });
}
