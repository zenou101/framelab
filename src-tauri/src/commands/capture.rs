use crate::fps::sidecar_path;
use crate::state::AppState;
use crate::types::*;
use tauri::State;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveFrames {
    pub displayed: Vec<f64>,
    pub presented: Vec<f64>,
    pub frame: Option<FrameTick>,
    pub status: FrameCaptureStatus,
    pub recorder: RecorderState,
    pub messages: Vec<String>,
}

#[tauri::command]
pub fn start_capture(state: State<'_, AppState>, pid: u32, process: String) -> Result<FrameCaptureStatus, String> {
    if pid == 0 {
        return Err("A process id is required".into());
    }
    if process.len() > 260 {
        return Err("Process name is too long".into());
    }
    let mut capture = state.capture.lock().map_err(|_| "Capture is unavailable".to_string())?;
    capture.start(pid, process, sidecar_path(state.paths.resource_dir.as_deref()))?;
    Ok(capture.status())
}

#[tauri::command]
pub fn stop_capture(state: State<'_, AppState>) -> FrameCaptureStatus {
    match state.capture.lock() {
        Ok(mut capture) => {
            capture.stop();
            capture.status()
        }
        Err(_) => FrameCaptureStatus { state: "unavailable".into(), ..Default::default() },
    }
}

#[tauri::command]
pub fn capture_status(state: State<'_, AppState>) -> FrameCaptureStatus {
    state.capture.lock().map(|c| c.status()).unwrap_or_default()
}

#[tauri::command]
pub fn live_frames(state: State<'_, AppState>, max: Option<usize>) -> LiveFrames {
    let capture = state.capture.lock().ok();
    let (displayed, presented) = capture.as_ref().map(|c| c.series_window(max.unwrap_or(2400).clamp(60, 12_000))).unwrap_or_default();
    LiveFrames {
        displayed,
        presented,
        frame: state.last_frame.lock().ok().and_then(|f| f.clone()),
        status: capture.as_ref().map(|c| c.status()).unwrap_or_default(),
        recorder: state.recorder.lock().map(|r| r.state()).unwrap_or_default(),
        messages: capture.as_ref().map(|c| c.messages()).unwrap_or_default(),
    }
}

#[tauri::command]
pub fn start_recording(state: State<'_, AppState>, game_id: Option<i64>, game_name: String) -> Result<RecorderState, String> {
    let name = game_name.trim().to_string();
    if name.is_empty() || name.chars().count() > 120 {
        return Err("A game name between 1 and 120 characters is required".into());
    }
    let (pid, process, primary_source) = {
        let capture = state.capture.lock().map_err(|_| "Capture is unavailable".to_string())?;
        let status = capture.status();
        if status.state != "running" {
            return Err("Start FPS capture for a running game before recording a benchmark".into());
        }
        let settings = state.settings.lock().map(|s| s.get()).unwrap_or_default();
        (status.pid.unwrap_or(0), status.process.unwrap_or_else(|| name.clone()), settings.primary_source)
    };
    if pid == 0 {
        return Err("No target process is attached".into());
    }
    let hardware_json = state
        .hardware
        .lock()
        .map(|h| serde_json::to_string(&*h).unwrap_or_default())
        .unwrap_or_default();
    let started = {
        let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
        let mut recorder = state.recorder.lock().map_err(|_| "Recorder is unavailable".to_string())?;
        recorder.start(&db, game_id, &name, &process, pid, &primary_source, &hardware_json)?
    };
    let _ = started;
    Ok(state.recorder.lock().map(|r| r.state()).unwrap_or_default())
}

#[tauri::command]
pub fn stop_recording(state: State<'_, AppState>) -> Result<Option<i64>, String> {
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    let mut recorder = state.recorder.lock().map_err(|_| "Recorder is unavailable".to_string())?;
    Ok(recorder.stop(&db, "completed").map(|(id, _)| id))
}

#[tauri::command]
pub fn reset_live_capture(state: State<'_, AppState>) -> FrameCaptureStatus {
    if let Ok(mut capture) = state.capture.lock() {
        let status = capture.status();
        if let Some(pid) = status.pid {
            let process = status.process.clone().unwrap_or_default();
            let _ = capture.start(pid, process, sidecar_path(state.paths.resource_dir.as_deref()));
        }
        capture.status()
    } else {
        FrameCaptureStatus::default()
    }
}
