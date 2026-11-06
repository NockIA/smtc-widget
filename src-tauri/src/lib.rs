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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app
                .get_webview_window("main")
                .expect("fenêtre 'main' introuvable");
            if let Err(e) = window::place_on_preferred_monitor(&main_window) {
                eprintln!("placement de la fenêtre échoué : {e}");
            }
            window::keep_on_top(main_window);

            media::start_listening(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![current_track, next_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
