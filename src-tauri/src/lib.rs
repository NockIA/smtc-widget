mod features;
mod shared;

use tauri::Manager;

use features::preferences::PreferencesStore;
use features::{appearance, context_menu, playback, volume, window};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            app.manage(PreferencesStore::load(&app.path().app_config_dir()?));

            let main_window = app
                .get_webview_window("main")
                .expect("fenêtre 'main' introuvable");

            let theme = app.state::<PreferencesStore>().snapshot().theme;
            if let Err(e) = appearance::apply_theme(&main_window, theme) {
                eprintln!("application du thème échouée : {e}");
            }

            window::init_placement(main_window.clone());
            appearance::watch_system_colors(main_window.clone());
            window::keep_on_top(main_window);

            playback::start_listening(app.handle().clone());

            Ok(())
        })
        .on_menu_event(context_menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            playback::commands::current_track,
            playback::commands::album_art,
            playback::commands::media_control,
            volume::commands::get_volume,
            volume::commands::set_volume,
            appearance::commands::get_system_colors,
            appearance::commands::refresh_system_colors,
            context_menu::commands::show_context_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
