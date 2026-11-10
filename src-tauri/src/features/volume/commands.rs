use tauri::State;

use super::VolumeWatcher;

/// Volume (0.0 à 1.0) de la source suivie, `None` si aucune session audio ne correspond.
#[tauri::command]
pub fn get_volume(watcher: State<'_, VolumeWatcher>) -> Option<f32> {
    watcher.volume()
}

#[tauri::command]
pub fn set_volume(watcher: State<'_, VolumeWatcher>, volume: f32) {
    watcher.set_volume(volume);
}
