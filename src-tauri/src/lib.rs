mod media;
mod window;

use tauri::Manager;

#[tauri::command]
async fn current_track() -> Result<Option<media::models::TrackInfo>, String> {
    media::smtc::get_current_track().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app
                .get_webview_window("main")
                .expect("fenêtre 'main' introuvable");
            window::keep_on_top(main_window);

            media::smtc::start_listening(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![current_track])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
