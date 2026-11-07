use tauri::State;

use super::models::{MediaAction, TrackInfo};
use super::services::{get_album_art, get_current_track, send_action};
use crate::features::audio_source::AudioSourceState;
use crate::shared::error::AppResult;

#[tauri::command]
pub async fn current_track(state: State<'_, AudioSourceState>) -> AppResult<Option<TrackInfo>> {
    Ok(get_current_track(state.selected().as_deref())?)
}

#[tauri::command]
pub async fn album_art(state: State<'_, AudioSourceState>) -> AppResult<Option<String>> {
    Ok(get_album_art(state.selected().as_deref())?)
}

#[tauri::command]
pub async fn media_control(
    state: State<'_, AudioSourceState>,
    action: MediaAction,
) -> AppResult<bool> {
    Ok(send_action(state.selected().as_deref(), action)?)
}
