use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use tauri::{WebviewWindow, WindowEvent};

use super::constants::SAVE_DEBOUNCE;
use super::placement::{current_placement, default_placement, monitor_key};
use super::preferences::WindowPreferences;

/// Une position identique à celle par défaut supprime la préférence de l'écran.
fn save_current_position(
    window: &WebviewWindow,
    path: &Path,
    prefs: &mut WindowPreferences,
) -> Result<(), Box<dyn Error>> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };
    let key = monitor_key(&monitor);
    let current = current_placement(window, &monitor)?;
    let default = default_placement(window, &monitor)?;

    prefs.last_monitor = Some(key.clone());
    if current.is_close_to(&default) {
        prefs.monitors.remove(&key);
    } else {
        prefs.monitors.insert(key, current);
    }
    prefs.save(path)?;
    Ok(())
}

pub fn track_moves(window: WebviewWindow, path: PathBuf, mut prefs: WindowPreferences) {
    let (tx, rx) = mpsc::channel::<()>();

    window.on_window_event(move |event| {
        if let WindowEvent::Moved(_) = event {
            let _ = tx.send(());
        }
    });

    thread::spawn(move || {
        while rx.recv().is_ok() {
            while rx.recv_timeout(SAVE_DEBOUNCE).is_ok() {}

            if let Err(e) = save_current_position(&window, &path, &mut prefs) {
                eprintln!("sauvegarde de la position échouée : {e}");
            }
        }
    });
}
