use crate::settings::Settings;
use crate::state::AppState;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().map(|s| s.get()).unwrap_or_default()
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let mut store = state.settings.lock().map_err(|_| "Settings unavailable".to_string())?;
    let applied = store.replace(settings)?;
    Ok(applied)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataStats {
    pub sessions: i64,
    pub games: i64,
    pub database_bytes: u64,
    pub data_dir: String,
}

#[tauri::command]
pub fn data_stats(state: State<'_, AppState>) -> DataStats {
    let (sessions, games) = state.db.lock().map(|db| db.counts()).unwrap_or((0, 0));
    let path = state.paths.data_dir.join("framelab.db");
    let database_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    DataStats {
        sessions,
        games,
        database_bytes,
        data_dir: state.paths.data_dir.to_string_lossy().to_string(),
    }
}

/// Applies retention now; returns how many sessions were removed.
#[tauri::command]
pub fn apply_retention(state: State<'_, AppState>) -> Result<usize, String> {
    let days = state.settings.lock().map(|s| s.get().retention_days).unwrap_or(0);
    if days == 0 {
        return Ok(0);
    }
    let cutoff = crate::sensors::now_ms() - (days as i64) * 86_400_000;
    let db = state.db.lock().map_err(|_| "Database unavailable".to_string())?;
    db.purge_older_than(cutoff)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let manager = app.autolaunch();
    let result = if enabled { manager.enable() } else { manager.disable() };
    result.map_err(|e| e.to_string())?;
    manager.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn autostart_enabled(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}
