use tauri::State;

use super::models::{MediaAction, TrackInfo};
use super::services::{get_album_art, get_current_track, send_action};
use crate::features::audio_source::AudioSourceState;
use crate::shared::error::{AppError, AppResult};

#[tauri::command]
pub async fn current_track(state: State<'_, AudioSourceState>) -> AppResult<Option<TrackInfo>> {
    get_current_track(state.selected().as_deref()).map_err(AppError::Smtc)
}

#[tauri::command]
pub async fn album_art(state: State<'_, AudioSourceState>) -> AppResult<Option<String>> {
    get_album_art(state.selected().as_deref()).map_err(AppError::Smtc)
}

#[tauri::command]
pub async fn media_control(
    state: State<'_, AudioSourceState>,
    action: MediaAction,
) -> AppResult<bool> {
    send_action(state.selected().as_deref(), action).map_err(AppError::Smtc)
}
