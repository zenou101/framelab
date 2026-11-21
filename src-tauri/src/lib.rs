pub mod analysis;
pub mod bench;
pub mod commands;
pub mod db;
pub mod fps;
pub mod games;
pub mod hardware;
pub mod probe;
pub mod sensors;
pub mod settings;
pub mod state;
pub mod stats;
pub mod types;

use bench::Sample;
use state::{AppPaths, AppState};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

fn spawn_sampler(handle: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        let state = handle.state::<AppState>();
        let sampling = state
            .settings
            .lock()
            .map(|store| store.get().sampling_ms)
            .unwrap_or(1000)
            .clamp(250, 5_000);

        let snapshot = match state.sampler.lock() {
            Ok(mut sampler) => sampler.snapshot(sampling),
            Err(_) => break,
        };
        let frame = state.capture.lock().ok().and_then(|capture| capture.tick());
        if let Ok(mut slot) = state.last_snapshot.lock() {
            *slot = Some(snapshot.clone());
        }
        if let Some(frame) = &frame {
            if let Ok(mut slot) = state.last_frame.lock() {
                *slot = Some(frame.clone());
            }
        }
        {
            let Ok(mut recorder) = state.recorder.lock() else { break };
            if recorder.state().active {
                if let Ok(capture) = state.capture.lock() {
                    let _ = capture.with_buffers(|buffers| recorder.ingest(buffers));
                }
                recorder.push_sample(Sample::from_snapshot(&snapshot));
                if let Ok(db) = state.db.lock() {
                    recorder.maybe_flush(&db);
                }
            }
        }

        if let Some(window) = handle.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let payload = serde_json::json!({ "snapshot": snapshot, "frame": frame });
                let _ = window.emit("framelab://tick", payload);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(sampling));
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();
            // A portable copy ships a marker file beside the executable and keeps
            // its database and settings in a data folder next to it instead of in
            // the user profile.
            let portable_root = std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(|dir| dir.to_path_buf()))
                .filter(|dir| dir.join("framelab-portable.txt").is_file());

            let data_dir = match &portable_root {
                Some(root) => root.join("data"),
                None => app
                    .path()
                    .app_data_dir()
                    .unwrap_or_else(|_| std::env::temp_dir().join("framelab")),
            };
            let config_dir = match &portable_root {
                Some(root) => root.join("data"),
                None => app.path().app_config_dir().unwrap_or_else(|_| data_dir.clone()),
            };
            let resource_dir = app.path().resource_dir().ok();
            let _ = std::fs::create_dir_all(&data_dir);
            let _ = std::fs::create_dir_all(&config_dir);

            let db = db::Db::open(&data_dir)
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            let recovered = db.recover_interrupted();
            let settings = settings::SettingsStore::load(&config_dir);
            let sampler = sensors::Sampler::new();
            let hardware = hardware::collect_hardware(&sampler);

            app.manage(AppState {
                sampler: Mutex::new(sampler),
                capture: Mutex::new(fps::Capture::new()),
                recorder: Mutex::new(bench::Recorder::default()),
                db: Mutex::new(db),
                settings: Mutex::new(settings),
                hardware: Mutex::new(hardware),
                last_snapshot: Mutex::new(None),
                last_frame: Mutex::new(None),
                paths: AppPaths { data_dir, config_dir, resource_dir },
                recovered: Mutex::new(recovered),
                started_at_ms: sensors::now_ms(),
            });

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            spawn_sampler(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::bootstrap,
            commands::system::live_snapshot,
            commands::system::live_frame,
            commands::system::hardware_info,
            commands::system::list_processes,
            commands::system::self_metrics,
            commands::system::restart_elevated,
            commands::system::open_data_dir,
            commands::capture::start_capture,
            commands::capture::stop_capture,
            commands::capture::capture_status,
            commands::capture::live_frames,
            commands::capture::start_recording,
            commands::capture::stop_recording,
            commands::capture::reset_live_capture,
            commands::data::list_sessions,
            commands::data::session_detail,
            commands::data::delete_session,
            commands::data::clear_sessions,
            commands::data::analyze,
            commands::data::export_session,
            commands::data::detect_games,
            commands::data::import_games,
            commands::data::list_games,
            commands::data::add_game_manual,
            commands::data::remove_game,
            commands::data::launch_game,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::data_stats,
            commands::settings::apply_retention,
            commands::settings::set_autostart,
            commands::settings::autostart_enabled
        ])
        .run(tauri::generate_context!())
        .expect("FrameLab failed to start");
}
