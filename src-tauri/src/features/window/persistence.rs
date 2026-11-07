use std::sync::mpsc;
use std::thread;

use tauri::{Manager, WebviewWindow, WindowEvent};

use super::constants::SAVE_DEBOUNCE;
use super::layout;
use super::placement::{current_placement, default_placement, monitor_key};
use crate::features::preferences::PreferencesStore;
use crate::shared::error::AppResult;

/// Une position identique à celle par défaut supprime la préférence de l'écran.
fn save_current_position(window: &WebviewWindow) -> AppResult<()> {
    if !layout::is_stable(window)? {
        return Ok(());
    }
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };
    let key = monitor_key(&monitor);
    let current = current_placement(window, &monitor)?;
    let default = default_placement(window, &monitor)?;

    window.state::<PreferencesStore>().update(|prefs| {
        prefs.last_monitor = Some(key.clone());
        if current.is_close_to(&default) {
            prefs.monitors.remove(&key);
        } else {
            prefs.monitors.insert(key, current);
        }
    })?;
    Ok(())
}

pub fn track_moves(window: WebviewWindow) {
    let (tx, rx) = mpsc::channel::<()>();

    window.on_window_event(move |event| {
        if let WindowEvent::Moved(_) = event {
            let _ = tx.send(());
        }
    });

    thread::spawn(move || {
        while rx.recv().is_ok() {
            while rx.recv_timeout(SAVE_DEBOUNCE).is_ok() {}

            if let Err(e) = save_current_position(&window) {
                eprintln!("sauvegarde de la position échouée : {e}");
            }
        }
    });
}
