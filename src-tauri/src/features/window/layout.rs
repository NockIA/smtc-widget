use std::sync::Mutex;
use std::thread;
use std::time::Instant;

use tauri::{Manager, WebviewWindow};

use super::constants::{LAYOUT_POLL_INTERVAL, LAYOUT_SETTLE};
use super::placement::{monitors_signature, place_on_preferred_monitor};
use crate::features::preferences::PreferencesStore;

static KNOWN_SIGNATURE: Mutex<String> = Mutex::new(String::new());
static SETTLING_UNTIL: Mutex<Option<Instant>> = Mutex::new(None);

/// Faux pendant qu'un écran vient d'être branché ou débranché : Windows déplace alors
/// la fenêtre lui-même, et ce déplacement ne doit pas être sauvegardé comme un choix.
pub fn is_stable(window: &WebviewWindow) -> tauri::Result<bool> {
    let settled = SETTLING_UNTIL
        .lock()
        .unwrap()
        .is_none_or(|until| Instant::now() >= until);
    Ok(settled && monitors_signature(window)? == *KNOWN_SIGNATURE.lock().unwrap())
}

/// Replace la fenêtre quand la liste des écrans change.
pub fn watch(window: WebviewWindow) {
    match monitors_signature(&window) {
        Ok(signature) => *KNOWN_SIGNATURE.lock().unwrap() = signature,
        Err(e) => {
            eprintln!("lecture des écrans échouée : {e}");
            return;
        }
    }

    thread::spawn(move || {
        loop {
            thread::sleep(LAYOUT_POLL_INTERVAL);

            let Ok(signature) = monitors_signature(&window) else {
                continue;
            };
            if signature == *KNOWN_SIGNATURE.lock().unwrap() {
                continue;
            }

            *SETTLING_UNTIL.lock().unwrap() = Some(Instant::now() + LAYOUT_SETTLE);
            *KNOWN_SIGNATURE.lock().unwrap() = signature;

            let prefs = window.state::<PreferencesStore>().snapshot();
            if let Err(e) = place_on_preferred_monitor(&window, &prefs) {
                eprintln!("replacement de la fenêtre échoué : {e}");
            }
        }
    });
}
