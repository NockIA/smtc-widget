use tauri::State;

use super::service;
use crate::features::audio_source::AudioSourceState;
use crate::shared::error::{AppError, AppResult};

#[tauri::command]
pub async fn get_volume(state: State<'_, AudioSourceState>) -> AppResult<Option<f32>> {
    service::get_volume(state.selected().as_deref()).map_err(AppError::Audio)
}

#[tauri::command]
pub async fn set_volume(state: State<'_, AudioSourceState>, volume: f32) -> AppResult<bool> {
    service::set_volume(state.selected().as_deref(), volume).map_err(AppError::Audio)
}
