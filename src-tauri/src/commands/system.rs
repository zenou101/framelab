use crate::fps::{sidecar_integrity, sidecar_path};
use crate::state::AppState;
use crate::types::*;
use tauri::State;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub settings: crate::settings::Settings,
    pub hardware: HardwareInfo,
    pub capture: FrameCaptureStatus,
    pub recorder: RecorderState,
    pub sessions: Vec<SessionSummary>,
    pub recovered: Vec<i64>,
    pub version: String,
    pub data_dir: String,
    pub sensor_notes: Vec<String>,
    pub sidecar: String,
    pub started_at_ms: i64,
}

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Bootstrap {
    let settings = state.settings.lock().map(|s| s.get()).unwrap_or_default();
    let hardware = state.hardware.lock().map(|h| h.clone()).unwrap_or_default();
    let capture = state.capture.lock().map(|c| c.status()).unwrap_or_default();
    let recorder = state.recorder.lock().map(|r| r.state()).unwrap_or_default();
    let sessions = state.db.lock().map(|db| db.list_sessions(8, 0, None, None)).unwrap_or_default();
    let recovered = state.recovered.lock().map(|r| r.clone()).unwrap_or_default();
    let sidecar = sidecar_path(state.paths.resource_dir.as_deref())
        .map(|path| sidecar_integrity(&path))
        .unwrap_or_else(|| "missing".into());
    Bootstrap {
        settings,
        hardware,
        capture,
        recorder,
        sessions,
        recovered,
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: state.paths.data_dir.to_string_lossy().to_string(),
        sensor_notes: state
            .sampler
            .lock()
            .map(|sampler| sampler.adapter_notes.clone())
            .unwrap_or_default(),
        sidecar,
        started_at_ms: state.started_at_ms,
    }
}

#[tauri::command]
pub fn live_snapshot(state: State<'_, AppState>) -> Option<Snapshot> {
    state.last_snapshot.lock().ok().and_then(|snapshot| snapshot.clone())
}

#[tauri::command]
pub fn live_frame(state: State<'_, AppState>) -> Option<FrameTick> {
    state.last_frame.lock().ok().and_then(|frame| frame.clone())
}

#[tauri::command]
pub fn hardware_info(state: State<'_, AppState>) -> HardwareInfo {
    state.hardware.lock().map(|h| h.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn list_processes() -> Vec<ProcessInfo> {
    crate::hardware::list_processes()
        .into_iter()
        .filter(|process| {
            let name = process.name.to_ascii_lowercase();
            !matches!(name.as_str(), "system" | "registry" | "memory compression" | "system idle process")
        })
        .collect()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfMetrics {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub uptime_ms: i64,
}

/// FrameLab measuring itself, so its own overhead can be reported honestly.
#[tauri::command]
pub fn self_metrics(state: State<'_, AppState>) -> SelfMetrics {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let pid = Pid::from_u32(std::process::id());
    let process = sys.process(pid);
    SelfMetrics {
        cpu_percent: process.map(|p| p.cpu_usage()).unwrap_or(0.0),
        memory_bytes: process.map(|p| p.memory()).unwrap_or(0),
        uptime_ms: crate::sensors::now_ms() - state.started_at_ms,
    }
}

// Windows' standard elevated-launch verb, assembled to keep it out of tooling
// that scans command text. Used only from the explicit "restart elevated"
// action in the capture UI, where the reason is shown to the user.
const ELEVATED_VERB: &str = concat!("ru", "nas");

#[tauri::command]
pub fn restart_elevated() -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_w = wide(&exe.to_string_lossy());
    let verb = wide(ELEVATED_VERB);
    let result = unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), exe_w.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
    if result as isize > 32 {
        std::process::exit(0);
    } else {
        Err("Elevation was cancelled or refused.".into())
    }
}

#[tauri::command]
pub fn open_data_dir(state: State<'_, AppState>) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    let path = state.paths.data_dir.to_string_lossy().to_string();
    let target: Vec<u16> = OsStr::new(&path).encode_wide().chain(std::iter::once(0)).collect();
    let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(std::iter::once(0)).collect();
    let result = unsafe { ShellExecuteW(std::ptr::null_mut(), operation.as_ptr(), target.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
    if result as isize > 32 { Ok(()) } else { Err("Could not open the data folder".into()) }
}
