use crate::analysis::analyze_session;
use crate::bench::decode_samples;
use crate::state::AppState;
use crate::stats;
use crate::types::*;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn list_sessions(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
    game_id: Option<i64>,
    search: Option<String>,
) -> Vec<SessionSummary> {
    let search = search.map(|s| s.chars().take(80).collect::<String>());
    state
        .db
        .lock()
        .map(|db| db.list_sessions(limit.unwrap_or(50), offset.unwrap_or(0), game_id, search.as_deref()))
        .unwrap_or_default()
}

#[tauri::command]
pub fn session_detail(state: State<'_, AppState>, id: i64, max_frames: Option<usize>) -> Option<SessionDetail> {
    let db = state.db.lock().ok()?;
    let summary = db.session(id)?;
    let displayed = db.load_frames(id, "displayed").unwrap_or_default();
    let presented = db.load_frames(id, "presented").unwrap_or_default();
    let samples = decode_samples(&db.load_samples(id));
    let displayed_stats = stats::analyze(&displayed, 2.0, 33.3);
    let presented_stats = stats::analyze(&presented, 2.0, 33.3);
    let histogram = stats::histogram(&displayed, 28);
    let cap = max_frames.unwrap_or(12_000).clamp(500, 60_000);
    let frames = if displayed.len() > cap {
        displayed[displayed.len() - cap..].to_vec()
    } else {
        displayed
    };
    Some(SessionDetail {
        summary,
        stats: SessionStats { displayed: displayed_stats, presented: presented_stats },
        histogram,
        sample_series: samples,
        frames,
    })
}

#[tauri::command]
pub fn delete_session(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "Database unavailable".to_string())?;
    db.delete_session(id)
}

#[tauri::command]
pub fn clear_sessions(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "Database unavailable".to_string())?;
    db.clear_sessions()
}

#[tauri::command]
pub fn analyze(state: State<'_, AppState>, id: i64) -> Option<AnalysisReport> {
    let thresholds = state.settings.lock().map(|s| s.get().thresholds).unwrap_or_default();
    let db = state.db.lock().ok()?;
    let frames = db.load_frames(id, "displayed").unwrap_or_default();
    if frames.is_empty() {
        return Some(AnalysisReport {
            findings: Vec::new(),
            not_evaluated: vec!["This session has no recorded frames.".into()],
            thresholds,
        });
    }
    let stats = stats::analyze(&frames, thresholds.spike_multiplier, thresholds.spike_floor_ms);
    let samples = decode_samples(&db.load_samples(id));
    Some(analyze_session(&frames, &stats, &samples, &thresholds))
}

fn resolve_export_path(path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path);
    if !candidate.is_absolute() {
        return Err("Choose an absolute export path".into());
    }
    let parent = candidate.parent().ok_or("The export path has no folder")?;
    let canonical = parent.canonicalize().map_err(|_| "That folder does not exist".to_string())?;
    let name = candidate.file_name().ok_or("The export path has no file name")?;
    Ok(canonical.join(name))
}

#[tauri::command]
pub fn export_session(state: State<'_, AppState>, id: i64, path: String, format: String) -> Result<String, String> {
    let target = resolve_export_path(&path)?;
    let db = state.db.lock().map_err(|_| "Database unavailable".to_string())?;
    let summary = db.session(id).ok_or("Session not found")?;
    let displayed = db.load_frames(id, "displayed").unwrap_or_default();
    let presented = db.load_frames(id, "presented").unwrap_or_default();
    let payload = if format == "json" {
        let detail = serde_json::json!({
            "schemaVersion": 1,
            "kind": "framelab.session",
            "exportedAtMs": crate::sensors::now_ms(),
            "session": summary,
            "displayedFrameTimesMs": displayed,
            "presentedFrameTimesMs": presented,
            "samples": decode_samples(&db.load_samples(id)),
        });
        serde_json::to_string_pretty(&detail).map_err(|e| e.to_string())?
    } else {
        let mut out = String::from("source,index,frame_time_ms,fps\n");
        for (index, frame) in displayed.iter().enumerate() {
            out.push_str(&format!("displayed,{index},{frame:.4},{:.3}\n", if *frame > 0.0 { 1000.0 / frame } else { 0.0 }));
        }
        for (index, frame) in presented.iter().enumerate() {
            out.push_str(&format!("presented,{index},{frame:.4},{:.3}\n", if *frame > 0.0 { 1000.0 / frame } else { 0.0 }));
        }
        out
    };
    std::fs::write(&target, payload).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedDto {
    pub name: String,
    pub source: String,
    pub launcher_id: String,
    pub exe_path: Option<String>,
    pub install_dir: Option<String>,
}

#[tauri::command]
pub fn detect_games() -> Vec<DetectedDto> {
    crate::games::detect_all()
        .into_iter()
        .map(|game| DetectedDto {
            name: game.name,
            source: game.source,
            launcher_id: game.launcher_id,
            exe_path: game.exe_path,
            install_dir: game.install_dir,
        })
        .collect()
}

#[tauri::command]
pub fn import_games(state: State<'_, AppState>) -> Vec<GameEntry> {
    let detected = crate::games::detect_all();
    let imported = state.db.lock().ok().map(|db| {
        for game in &detected {
            let _ = db.insert_game(&game.name, game.exe_path.as_deref(), &game.source, Some(&game.launcher_id), None);
        }
        db.games()
    });
    imported.unwrap_or_default()
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Vec<GameEntry> {
    state.db.lock().map(|db| db.games()).unwrap_or_default()
}

#[tauri::command]
pub fn add_game_manual(state: State<'_, AppState>, path: String) -> Result<Vec<GameEntry>, String> {
    let canonical = std::fs::canonicalize(&path).map_err(|_| "That file does not exist".to_string())?;
    if !canonical.is_file() {
        return Err("That path is not a file".into());
    }
    if canonical.extension().map(|e| !e.eq_ignore_ascii_case("exe")).unwrap_or(true) {
        return Err("Choose a .exe file".into());
    }
    let name = canonical
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .ok_or("Could not read the executable name")?;
    let db = state.db.lock().map_err(|_| "Database unavailable".to_string())?;
    db.insert_game(&name, Some(&canonical.to_string_lossy()), "manual", None, None)?;
    Ok(db.games())
}

#[tauri::command]
pub fn remove_game(state: State<'_, AppState>, id: i64) -> Result<Vec<GameEntry>, String> {
    let db = state.db.lock().map_err(|_| "Database unavailable".to_string())?;
    db.delete_game(id)?;
    Ok(db.games())
}

#[tauri::command]
pub fn launch_game(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let games = state.db.lock().map(|db| db.games()).unwrap_or_default();
    let game = games.into_iter().find(|g| g.id == id).ok_or("That game is not in your library")?;
    crate::games::launch(game.exe_path.as_deref(), game.launcher_id.as_deref())
}
