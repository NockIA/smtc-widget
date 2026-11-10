use tauri::State;

use super::artwork::get_album_art;
use super::controls::send_action;
use super::models::{MediaAction, TrackInfo};
use super::track::get_current_track;
use crate::features::audio_source::AudioSourceState;
use crate::shared::error::{AppError, AppResult};

#[tauri::command]
pub async fn current_track(state: State<'_, AudioSourceState>) -> AppResult<Option<TrackInfo>> {
    get_current_track(state.selected().as_deref())
        .await
        .map_err(AppError::Smtc)
}

/// Décodage et réduction de l'image : travail lourd, sur le pool dédié aux appels bloquants.
#[tauri::command]
pub async fn album_art(state: State<'_, AudioSourceState>) -> AppResult<Option<String>> {
    let selected = state.selected();
    tauri::async_runtime::spawn_blocking(move || get_album_art(selected.as_deref()))
        .await?
        .map_err(AppError::Smtc)
}

#[tauri::command]
pub async fn media_control(state: State<'_, AudioSourceState>, action: MediaAction) -> AppResult<bool> {
    send_action(state.selected().as_deref(), action)
        .await
        .map_err(AppError::Smtc)
}
