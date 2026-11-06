mod media;
mod window;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app
                .get_webview_window("main")
                .expect("fenêtre 'main' introuvable");
            window::keep_on_top(main_window);

            // Test temporaire : lit le morceau en cours une fois, hors du thread principal.
            std::thread::spawn(|| {
                println!("{:?}", media::smtc::get_current_track());
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
