use serde::{Deserialize, Serialize};

/// A single measured value with provenance. `available == false` is never
/// rendered as zero: the UI shows the reason instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub value: Option<f64>,
    pub available: bool,
    pub source: String,
    pub method: Option<String>,
    pub reason: Option<String>,
}

impl Metric {
    pub fn ok(source: impl Into<String>, value: f64) -> Self {
        Self { value: Some(value), available: true, source: source.into(), method: None, reason: None }
    }

    pub fn ok_method(source: impl Into<String>, value: f64, method: impl Into<String>) -> Self {
        Self {
            value: Some(value),
            available: true,
            source: source.into(),
            method: Some(method.into()),
            reason: None,
        }
    }

    pub fn unavailable(source: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            value: None,
            available: false,
            source: source.into(),
            method: None,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub ts_ms: i64,
    pub sampling_ms: u64,
    pub cpu_percent: Metric,
    pub cpu_clock_mhz: Metric,
    pub cpu_per_core: Vec<f64>,
    pub ram_percent: Metric,
    pub ram_used_bytes: Metric,
    pub ram_total_bytes: Metric,
    pub gpu_percent: Metric,
    pub vram_used_bytes: Metric,
    pub vram_total_bytes: Metric,
    pub disk_bytes_per_sec: Metric,
    pub net_bytes_per_sec: Metric,
    pub cpu_temp_c: Metric,
    pub gpu_temp_c: Metric,
    pub frame_source_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub mount: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub gpu_name: String,
    pub vram_total_bytes: Option<u64>,
    pub ram_total_bytes: u64,
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub disks: Vec<DiskInfo>,
    pub display: Option<DisplayInfo>,
    pub adapter_luid: Option<String>,
    pub notes: Vec<String>,
    pub collected_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe_path: Option<String>,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub is_foreground: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FrameSource {
    Displayed,
    Presented,
}

impl FrameSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Displayed => "displayed",
            Self::Presented => "presented",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameTick {
    pub ts_ms: i64,
    /// Instantaneous FPS derived from on-screen cadence (MsBetweenDisplayChange).
    pub displayed_fps: Option<f64>,
    /// Instantaneous FPS derived from present cadence (MsBetweenPresents).
    pub presented_fps: Option<f64>,
    pub displayed_frame_ms: Option<f64>,
    pub presented_frame_ms: Option<f64>,
    pub pid: u32,
    pub process: String,
    pub present_mode: Option<String>,
    pub runtime: Option<String>,
    pub sync_interval: Option<i64>,
    pub captured_displayed: u64,
    pub captured_presented: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrameCaptureStatus {
    pub state: String,
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub message: Option<String>,
    pub elevation: String,
    pub sidecar_present: bool,
    pub sidecar_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: i64,
    pub game_id: Option<i64>,
    pub game_name: String,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub duration_ms: i64,
    pub status: String,
    pub primary_source: String,
    pub frames_displayed: Option<i64>,
    pub frames_presented: Option<i64>,
    pub avg_fps_displayed: Option<f64>,
    pub avg_fps_presented: Option<f64>,
    pub median_fps_displayed: Option<f64>,
    pub p1_low_displayed: Option<f64>,
    pub p01_low_displayed: Option<f64>,
    pub p1_low_presented: Option<f64>,
    pub stutter_count: Option<i64>,
    pub recovered: bool,
    pub hardware_json: Option<String>,
    pub meta_json: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub summary: SessionSummary,
    pub stats: SessionStats,
    pub histogram: Vec<HistogramBin>,
    pub sample_series: SampleSeries,
    pub frames: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub displayed: FrameStats,
    pub presented: FrameStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrameStats {
    pub frames: i64,
    pub duration_ms: f64,
    pub avg_fps: Option<f64>,
    pub median_fps: Option<f64>,
    pub p1_low: Option<f64>,
    pub p01_low: Option<f64>,
    pub avg_frame_ms: Option<f64>,
    pub median_frame_ms: Option<f64>,
    pub p95_frame_ms: Option<f64>,
    pub p99_frame_ms: Option<f64>,
    pub min_frame_ms: Option<f64>,
    pub max_frame_ms: Option<f64>,
    pub cv: Option<f64>,
    pub stutter_count: i64,
    pub stutter_ms_total: f64,
    pub insufficient_samples: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistogramBin {
    pub from_ms: f64,
    pub to_ms: f64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SampleSeries {
    pub t_ms: Vec<i64>,
    pub cpu: Vec<f32>,
    pub gpu: Vec<f32>,
    pub ram: Vec<f32>,
    pub vram: Vec<f32>,
    pub cpu_temp: Vec<f32>,
    pub gpu_temp: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecorderState {
    pub active: bool,
    pub session_id: Option<i64>,
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub started_at_ms: Option<i64>,
    pub displayed_frames: u64,
    pub presented_frames: u64,
    pub persisted_frames: u64,
    pub backpressure: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub code: String,
    pub severity: String,
    pub title: String,
    pub explanation: String,
    pub next_step: String,
    pub confidence: String,
    pub evidence: Vec<String>,
    pub heuristic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisReport {
    pub findings: Vec<Finding>,
    pub not_evaluated: Vec<String>,
    pub thresholds: Thresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEntry {
    pub id: i64,
    pub name: String,
    pub exe_path: Option<String>,
    pub source: String,
    pub launcher_id: Option<String>,
    pub icon_path: Option<String>,
    pub created_at_ms: i64,
    pub session_count: i64,
    pub best_avg_fps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Thresholds {
    pub pacing_cv: f64,
    pub pacing_p99_ratio: f64,
    pub spike_multiplier: f64,
    pub spike_floor_ms: f64,
    pub spike_cluster_window_ms: i64,
    pub spike_cluster_min: u32,
    pub ram_pressure_pct: f64,
    pub vram_pressure_pct: f64,
    pub gpu_dip_points: f64,
    pub single_core_pct: f64,
    pub gpu_underutilized_pct: f64,
    pub background_cpu_pct: f64,
    pub thermal_limit_c: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            pacing_cv: 0.25,
            pacing_p99_ratio: 2.5,
            spike_multiplier: 2.0,
            spike_floor_ms: 33.3,
            spike_cluster_window_ms: 500,
            spike_cluster_min: 5,
            ram_pressure_pct: 90.0,
            vram_pressure_pct: 92.0,
            gpu_dip_points: 25.0,
            single_core_pct: 90.0,
            gpu_underutilized_pct: 80.0,
            background_cpu_pct: 15.0,
            thermal_limit_c: 95.0,
        }
    }
}
