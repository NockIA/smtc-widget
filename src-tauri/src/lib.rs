mod context_menu;
mod media;
mod preferences;
mod system_colors;
mod theme;
mod window;

use tauri::Manager;

use preferences::PreferencesStore;

#[tauri::command]
async fn current_track(
    state: tauri::State<'_, media::MediaState>,
) -> Result<Option<media::models::TrackInfo>, String> {
    media::get_current_track(state.selected().as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn media_control(
    state: tauri::State<'_, media::MediaState>,
    action: media::MediaAction,
) -> Result<bool, String> {
    media::send_action(state.selected().as_deref(), action).map_err(|e| e.to_string())
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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
        }))
        .setup(|app| {
            app.manage(PreferencesStore::load(&app.path().app_config_dir()?));

            let main_window = app
                .get_webview_window("main")
                .expect("fenêtre 'main' introuvable");

            let theme = app.state::<PreferencesStore>().snapshot().theme;
            if let Err(e) = theme::apply_theme(&main_window, theme) {
                eprintln!("application du thème échouée : {e}");
            }

            window::init_placement(main_window.clone());
            system_colors::watch(main_window.clone());
            window::keep_on_top(main_window);

            media::start_listening(app.handle().clone());

            Ok(())
        })
        .on_menu_event(context_menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            current_track,
            media_control,
            get_volume,
            set_volume,
            system_colors::get_system_colors,
            context_menu::show_context_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
