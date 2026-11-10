mod features;
mod shared;

use tauri::Manager;

use features::preferences::PreferencesStore;
use features::{appearance, context_menu, playback, volume, window};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_panic_hook();

    let result = tauri::Builder::default()
        .plugin(logging_plugin())
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

            let main_window = app.get_webview_window("main").ok_or("fenêtre 'main' introuvable")?;

            let theme = app.state::<PreferencesStore>().snapshot().theme;
            if let Err(e) = appearance::apply_theme(&main_window, theme) {
                log::error!("application du thème a échoué : {e}");
            }

            window::init_placement(main_window.clone());
            appearance::watch_system_colors(main_window.clone());
            window::keep_on_top(main_window);

            app.manage(volume::VolumeWatcher::start(app.handle().clone()));
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
        .run(tauri::generate_context!());

    if let Err(e) = result {
        log::error!("arrêt de l'application sur erreur : {e}");
        log::logger().flush();
        std::process::exit(1);
    }
}

/// Fichier de log dans le dossier de logs de l'app (un fichier + sa rotation), console en plus en debug.
fn logging_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    let mut builder = tauri_plugin_log::Builder::new()
        .level(level)
        .max_file_size(512 * 1024)
        .rotation_strategy(RotationStrategy::KeepOne)
        .target(Target::new(TargetKind::LogDir { file_name: None }));
    if cfg!(debug_assertions) {
        builder = builder.target(Target::new(TargetKind::Stdout));
    }
    builder.build()
}

/// Le profil release est en `panic = "abort"` : sans ce hook, la cause d'un crash serait perdue.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panique : {info}");
        log::logger().flush();
        default_hook(info);
    }));
}
