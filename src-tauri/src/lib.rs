mod media;
mod window;

use tauri::Manager;

#[tauri::command]
async fn current_track(
    state: tauri::State<'_, media::MediaState>,
) -> Result<Option<media::models::TrackInfo>, String> {
    media::get_current_track(state.selected().as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn next_session(state: tauri::State<'_, media::MediaState>) {
    state.request_next_session();
}

#[tauri::command]
async fn media_control(
    state: tauri::State<'_, media::MediaState>,
    action: media::MediaAction,
) -> Result<bool, String> {
    media::send_action(state.selected().as_deref(), action).map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// Volume (0.0 à 1.0) de la source courante, `None` si aucune session audio ne correspond.
#[tauri::command]
async fn get_volume(state: tauri::State<'_, media::MediaState>) -> Result<Option<f32>, String> {
    media::get_volume(state.selected().as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_volume(
    state: tauri::State<'_, media::MediaState>,
    volume: f32,
) -> Result<bool, String> {
    media::set_volume(state.selected().as_deref(), volume).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app
                .get_webview_window("main")
                .expect("fenêtre 'main' introuvable");
            window::init_placement(
                main_window.clone(),
                window::preferences_path(app.path().app_config_dir()?),
            );
            window::keep_on_top(main_window);

            media::start_listening(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            current_track,
            next_session,
            media_control,
            get_volume,
            set_volume,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
