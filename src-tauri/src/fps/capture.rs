use super::parser::{CsvParser, ParsedFrame};
use crate::types::{FrameCaptureStatus, FrameTick};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const MAX_BUFFER: usize = 120_000;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Default)]
pub struct FrameBuffers {
    pub displayed: VecDeque<f64>,
    pub presented: VecDeque<f64>,
    pub captured_displayed: u64,
    pub captured_presented: u64,
    pub dropped: u64,
    pub present_mode: Option<String>,
    pub runtime: Option<String>,
    pub sync_interval: Option<i64>,
    pub pid: u32,
    pub process: String,
    pub last_displayed_ms: Option<f64>,
    pub last_presented_ms: Option<f64>,
    pub messages: VecDeque<String>,
}

impl FrameBuffers {
    pub fn push(&mut self, frame: &ParsedFrame) {
        if let Some(ms) = frame.displayed_ms.filter(|v| *v > 0.0) {
            self.captured_displayed += 1;
            self.last_displayed_ms = Some(ms);
            if self.displayed.len() >= MAX_BUFFER {
                self.displayed.pop_front();
                self.dropped += 1;
            }
            self.displayed.push_back(ms);
        }
        if let Some(ms) = frame.presented_ms.filter(|v| *v > 0.0) {
            self.captured_presented += 1;
            self.last_presented_ms = Some(ms);
            if self.presented.len() >= MAX_BUFFER {
                self.presented.pop_front();
                self.dropped += 1;
            }
            self.presented.push_back(ms);
        }
        if frame.present_mode.is_some() {
            self.present_mode = frame.present_mode.clone();
        }
        if frame.runtime.is_some() {
            self.runtime = frame.runtime.clone();
        }
        if frame.sync_interval.is_some() {
            self.sync_interval = frame.sync_interval;
        }
    }

    pub fn push_message(&mut self, message: String) {
        if self.messages.len() >= 20 {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    pub fn tail(&self, displayed: bool) -> Option<(f64, f64)> {
        let queue = if displayed { &self.displayed } else { &self.presented };
        if queue.is_empty() {
            return None;
        }
        let take = queue.len().min(45);
        let sum: f64 = queue.iter().rev().take(take).sum();
        if sum <= 0.0 {
            return None;
        }
        let mean = sum / take as f64;
        Some((mean, 1000.0 / mean))
    }

    pub fn series(&self, displayed: bool) -> Vec<f64> {
        let queue = if displayed { &self.displayed } else { &self.presented };
        queue.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.displayed.clear();
        self.presented.clear();
        self.captured_displayed = 0;
        self.captured_presented = 0;
        self.dropped = 0;
        self.last_displayed_ms = None;
        self.last_presented_ms = None;
    }
}

pub(crate) struct Shared {
    status: FrameCaptureStatus,
    buffers: FrameBuffers,
}

/// PresentMon writes UTF-16LE when its stdout is a pipe. A UTF-8 line reader
/// stops at the first byte, so every line is decoded explicitly.
fn decode_line(bytes: &[u8]) -> String {
    let has_bom = bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE;
    let looks_utf16 = has_bom
        || (bytes.len() >= 4 && bytes.iter().skip(1).take(64).filter(|byte| **byte == 0).count() > 8);
    let text = if looks_utf16 {
        let body = if has_bom { &bytes[2..] } else { bytes };
        let units: Vec<u16> = body
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    };
    text.trim_end_matches(['\r', '\n']).to_string()
}

pub struct Capture {
    shared: Arc<Mutex<Shared>>,
    stop: Arc<AtomicBool>,
    child: Option<Child>,
    sidecar: Option<PathBuf>,
}

pub fn sidecar_path(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = resource_dir {
        // Bundled resources keep their configured path ("binaries/presentmon/*").
        candidates.push(
            dir.join("binaries")
                .join("presentmon")
                .join("PresentMon.exe"),
        );
        candidates.push(dir.join("presentmon").join("PresentMon.exe"));
        candidates.push(dir.join("PresentMon.exe"));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/presentmon/PresentMon.exe"));
    candidates.push(PathBuf::from("src-tauri/binaries/presentmon/PresentMon.exe"));
    candidates.into_iter().find(|path| path.is_file())
}

/// Integrity check against the SHA-256 recorded next to the binary at build time.
pub fn sidecar_integrity(path: &Path) -> String {
    let manifest = path.with_file_name("SHA256.txt");
    let Ok(expected) = std::fs::read_to_string(&manifest) else {
        return "unverified".into();
    };
    let expected = expected.trim().to_ascii_lowercase();
    if expected.is_empty() {
        return "unverified".into();
    }
    match std::fs::read(path) {
        Ok(bytes) => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());
            if actual == expected { "verified".into() } else { "mismatch".into() }
        }
        Err(_) => "unreadable".into(),
    }
}

impl Capture {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                status: FrameCaptureStatus {
                    state: "idle".into(),
                    elevation: "unknown".into(),
                    ..Default::default()
                },
                buffers: FrameBuffers::default(),
            })),
            stop: Arc::new(AtomicBool::new(false)),
            child: None,
            sidecar: None,
        }
    }

    pub fn shared(&self) -> Arc<Mutex<Shared>> {
        Arc::clone(&self.shared)
    }

    pub fn status(&self) -> FrameCaptureStatus {
        self.shared.lock().map(|s| s.status.clone()).unwrap_or_default()
    }

    pub fn is_running(&self) -> bool {
        self.shared.lock().map(|s| s.status.state == "running" || s.status.state == "starting").unwrap_or(false)
    }

    pub fn tick(&self) -> Option<FrameTick> {
        let guard = self.shared.lock().ok()?;
        if guard.status.state != "running" {
            return None;
        }
        let buffers = &guard.buffers;
        let displayed = buffers.tail(true);
        let presented = buffers.tail(false);
        Some(FrameTick {
            ts_ms: crate::sensors::now_ms(),
            displayed_fps: displayed.map(|(_, fps)| fps),
            presented_fps: presented.map(|(_, fps)| fps),
            displayed_frame_ms: displayed.map(|(ms, _)| ms),
            presented_frame_ms: presented.map(|(ms, _)| ms),
            pid: buffers.pid,
            process: buffers.process.clone(),
            present_mode: buffers.present_mode.clone(),
            runtime: buffers.runtime.clone(),
            sync_interval: buffers.sync_interval,
            captured_displayed: buffers.captured_displayed,
            captured_presented: buffers.captured_presented,
        })
    }

    pub fn start(&mut self, pid: u32, process: String, sidecar: Option<PathBuf>) -> Result<(), String> {
        self.stop_internal("replaced");
        let Some(path) = sidecar else {
            self.set_status("unavailable", None, Some(process), Some(
                "PresentMon helper not found in the application resources. FPS capture is unavailable until it is present.".into(),
            ));
            return Err("PresentMon helper not found".into());
        };
        let integrity = sidecar_integrity(&path);
        if integrity == "mismatch" {
            self.set_status("unavailable", Some(pid), Some(process), Some(
                "PresentMon helper failed its SHA-256 integrity check and was not started.".into(),
            ));
            return Err("PresentMon integrity check failed".into());
        }
        let mut command = Command::new(&path);
        command
            .arg("--process_id").arg(pid.to_string())
            .arg("--output_stdout")
            .arg("--no_console_stats")
            .arg("--exclude_dropped")
            .arg("--terminate_on_proc_exit")
            .arg("--stop_existing_session")
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|e| {
            self.set_status("unavailable", Some(pid), Some(process.clone()), Some(format!("Could not start PresentMon: {e}")));
            e.to_string()
        })?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        self.stop.store(false, Ordering::SeqCst);
        {
            let mut guard = self.shared.lock().map_err(|_| "capture lock poisoned".to_string())?;
            guard.buffers.clear();
            guard.buffers.pid = pid;
            guard.buffers.process = process.clone();
            guard.status = FrameCaptureStatus {
                state: "starting".into(),
                pid: Some(pid),
                process: Some(process),
                message: Some("Waiting for the first frame from the target process…".into()),
                elevation: "unknown".into(),
                sidecar_present: true,
                sidecar_path: Some(path.to_string_lossy().to_string()),
            };
            if integrity == "verified" {
                guard.buffers.push_message("PresentMon helper integrity verified (SHA-256).".into());
            }
        }
        if let Some(stdout) = stdout {
            let shared = Arc::clone(&self.shared);
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut buffer = Vec::new();
                let mut parser: Option<CsvParser> = None;
                let mut saw_header = false;
                loop {
                    buffer.clear();
                    match reader.read_until(b'\n', &mut buffer) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(_) => break,
                    }
                    let line = decode_line(&buffer);
                    if line.is_empty() {
                        continue;
                    }
                    if parser.is_none() {
                        // PresentMon prints warnings on stdout before the CSV
                        // header, so every line is offered to the header parser.
                        parser = CsvParser::header(&line);
                        if parser.is_some() {
                            saw_header = true;
                        } else if let Ok(mut guard) = shared.lock() {
                            guard.buffers.push_message(line);
                        }
                        continue;
                    }
                    if let Some(parsed) = parser.as_ref().and_then(|p| p.parse(&line)) {
                        if let Ok(mut guard) = shared.lock() {
                            guard.buffers.push(&parsed);
                            if guard.status.state == "starting" {
                                guard.status.state = "running".into();
                                guard.status.message = None;
                            }
                        }
                    }
                }
                if let Ok(mut guard) = shared.lock() {
                    if guard.status.state == "running" || guard.status.state == "starting" {
                        guard.status.state = "stopped".into();
                        guard.status.message = Some(if saw_header {
                            "Capture ended: the target process exited.".into()
                        } else {
                            "PresentMon produced no CSV header. Check that the target process renders frames.".into()
                        });
                    }
                }
            });
        }
        if let Some(stderr) = stderr {
            let shared = Arc::clone(&self.shared);
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stderr);
                let mut buffer = Vec::new();
                loop {
                    buffer.clear();
                    match reader.read_until(b'\n', &mut buffer) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(_) => break,
                    }
                    let line = decode_line(&buffer);
                    if line.is_empty() {
                        continue;
                    }
                    let lower = line.to_ascii_lowercase();
                    if let Ok(mut guard) = shared.lock() {
                        guard.buffers.push_message(line.clone());
                        if lower.contains("access") && lower.contains("denied") {
                            guard.status.state = "denied".into();
                            guard.status.elevation = "denied".into();
                            guard.status.message = Some(
                                "Windows denied the ETW trace session. Run FrameLab as administrator, or add this account to the local Performance Log Users group, then retry.".into(),
                            );
                        } else if guard.status.state == "starting" {
                            guard.status.message = Some(line);
                        }
                    }
                }
            });
        }
        self.child = Some(child);
        self.sidecar = Some(path);
        Ok(())
    }

    fn set_status(&mut self, state: &str, pid: Option<u32>, process: Option<String>, message: Option<String>) {
        if let Ok(mut guard) = self.shared.lock() {
            guard.status.state = state.into();
            guard.status.pid = pid;
            guard.status.process = process;
            guard.status.message = message;
            guard.status.sidecar_present = self.sidecar.as_ref().map(|p| p.is_file()).unwrap_or(false);
            guard.status.sidecar_path = self.sidecar.as_ref().map(|p| p.to_string_lossy().to_string());
        }
    }

    fn stop_internal(&mut self, reason: &str) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Ok(mut guard) = self.shared.lock() {
            if guard.status.state == "running" || guard.status.state == "starting" {
                guard.status.state = "stopped".into();
                guard.status.message = Some(format!("Capture stopped ({reason})."));
            }
        }
    }

    pub fn stop(&mut self) {
        self.stop_internal("user");
        if let Ok(mut guard) = self.shared.lock() {
            if guard.status.state != "denied" && guard.status.state != "unavailable" {
                guard.status.state = "idle".into();
                guard.status.pid = None;
                guard.status.process = None;
            }
        }
    }
}

impl Default for Capture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture {
    /// Borrow the live buffers without copying them (the recorder ingests
    /// straight from this view inside the sampling thread).
    pub fn with_buffers<R>(&self, f: impl FnOnce(&FrameBuffers) -> R) -> Option<R> {
        self.shared.lock().ok().map(|guard| f(&guard.buffers))
    }

    pub fn series_window(&self, max: usize) -> (Vec<f64>, Vec<f64>) {
        self.shared
            .lock()
            .map(|guard| {
                let take = |queue: &VecDeque<f64>| {
                    let start = queue.len().saturating_sub(max);
                    queue.iter().skip(start).copied().collect::<Vec<f64>>()
                };
                (take(&guard.buffers.displayed), take(&guard.buffers.presented))
            })
            .unwrap_or_default()
    }

    pub fn messages(&self) -> Vec<String> {
        self.shared
            .lock()
            .map(|guard| guard.buffers.messages.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf16_presentmon_stdout() {
        let header = "Application,ProcessID,MsBetweenPresents,MsBetweenDisplayChange\r\n";
        let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
        for unit in header.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let line = decode_line(&bytes);
        assert_eq!(line, header.trim_end());
        assert!(CsvParser::header(&line).is_some(), "UTF-16 header must parse");
    }

    #[test]
    fn decodes_plain_utf8_lines() {
        assert_eq!(decode_line(b"a,b,c\r\n"), "a,b,c");
        assert_eq!(decode_line(b""), "");
    }

    #[test]
    fn warnings_before_header_are_not_data() {
        let warning = "warning: only one of the following options may be used";
        assert!(CsvParser::header(warning).is_none());
    }

    #[test]
    fn finds_sidecar_in_bundled_resource_layout() {
        // Bundles keep the configured resource path, so the helper sits under
        // binaries/presentmon beside the executable.
        let root = std::env::temp_dir().join(format!("framelab-sidecar-{}", std::process::id()));
        let dir = root.join("binaries").join("presentmon");
        std::fs::create_dir_all(&dir).expect("create temp resource dir");
        let exe = dir.join("PresentMon.exe");
        std::fs::write(&exe, b"stub").expect("write stub sidecar");

        let found = sidecar_path(Some(&root));
        assert_eq!(found.as_deref(), Some(exe.as_path()));

        let _ = std::fs::remove_dir_all(&root);
    }
}
