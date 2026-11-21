use crate::types::Thresholds;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct OverlaySettings {
    pub enabled: bool,
    pub position: String,
    pub opacity: f64,
    pub compact: bool,
    pub show_fps: bool,
    pub show_frame_time: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_gpu_temp: bool,
    pub show_ram: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            position: "top-left".into(),
            opacity: 0.85,
            compact: true,
            show_fps: true,
            show_frame_time: true,
            show_cpu: true,
            show_gpu: true,
            show_gpu_temp: true,
            show_ram: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct EnabledMetrics {
    pub cpu: bool,
    pub gpu: bool,
    pub ram: bool,
    pub vram: bool,
    pub disk: bool,
    pub network: bool,
    pub temperatures: bool,
}

impl Default for EnabledMetrics {
    fn default() -> Self {
        Self { cpu: true, gpu: true, ram: true, vram: true, disk: true, network: true, temperatures: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Settings {
    pub theme: String,
    pub accent: String,
    pub compact: bool,
    pub reduce_motion: bool,
    pub sampling_ms: u64,
    pub background_monitoring: bool,
    pub primary_source: String,
    pub enabled_metrics: EnabledMetrics,
    pub export_format: String,
    pub retention_days: u32,
    pub diagnostics_enabled: bool,
    pub launch_on_startup: bool,
    pub minimize_to_tray: bool,
    pub native_frame: bool,
    pub overlay: OverlaySettings,
    pub thresholds: Thresholds,
    pub sidebar_collapsed: bool,
    /// Version identifier of the in-app legal notice the user acknowledged.
    /// Empty means the notice has not been shown yet.
    pub legal_accepted_version: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            accent: "blue".into(),
            compact: false,
            reduce_motion: false,
            sampling_ms: 1000,
            background_monitoring: false,
            primary_source: "displayed".into(),
            enabled_metrics: EnabledMetrics::default(),
            export_format: "csv".into(),
            retention_days: 0,
            diagnostics_enabled: true,
            launch_on_startup: false,
            minimize_to_tray: false,
            native_frame: false,
            overlay: OverlaySettings::default(),
            thresholds: Thresholds::default(),
            sidebar_collapsed: false,
            legal_accepted_version: String::new(),
        }
    }
}

const ACCENTS: [&str; 6] = ["blue", "violet", "teal", "amber", "rose", "lime"];

impl Settings {
    /// Clamp every value that drives a system call so IPC input can never push
    /// sampling below a sane floor or write outside validated ranges.
    pub fn validate(&mut self) {
        if !matches!(self.theme.as_str(), "dark" | "light" | "system") {
            self.theme = "dark".into();
        }
        if !ACCENTS.contains(&self.accent.as_str()) {
            self.accent = "blue".into();
        }
        if !matches!(self.primary_source.as_str(), "displayed" | "presented") {
            self.primary_source = "displayed".into();
        }
        if !matches!(self.export_format.as_str(), "csv" | "json") {
            self.export_format = "csv".into();
        }
        self.sampling_ms = self.sampling_ms.clamp(250, 5_000);
        self.legal_accepted_version = self.legal_accepted_version.chars().take(32).collect();
        self.retention_days = self.retention_days.min(3_650);
        self.overlay.opacity = self.overlay.opacity.clamp(0.3, 1.0);
        if !matches!(self.overlay.position.as_str(), "top-left" | "top-right" | "bottom-left" | "bottom-right") {
            self.overlay.position = "top-left".into();
        }
        let t = &mut self.thresholds;
        t.pacing_cv = t.pacing_cv.clamp(0.05, 2.0);
        t.pacing_p99_ratio = t.pacing_p99_ratio.clamp(1.1, 10.0);
        t.spike_multiplier = t.spike_multiplier.clamp(1.2, 6.0);
        t.spike_floor_ms = t.spike_floor_ms.clamp(8.0, 500.0);
        t.spike_cluster_window_ms = t.spike_cluster_window_ms.clamp(50, 10_000);
        t.spike_cluster_min = t.spike_cluster_min.clamp(2, 500);
        t.ram_pressure_pct = t.ram_pressure_pct.clamp(50.0, 100.0);
        t.vram_pressure_pct = t.vram_pressure_pct.clamp(50.0, 100.0);
        t.gpu_dip_points = t.gpu_dip_points.clamp(5.0, 90.0);
        t.single_core_pct = t.single_core_pct.clamp(40.0, 100.0);
        t.gpu_underutilized_pct = t.gpu_underutilized_pct.clamp(10.0, 99.0);
        t.background_cpu_pct = t.background_cpu_pct.clamp(1.0, 100.0);
        t.thermal_limit_c = t.thermal_limit_c.clamp(50.0, 120.0);
    }
}

pub struct SettingsStore {
    path: PathBuf,
    current: Settings,
}

impl SettingsStore {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("settings.json");
        let current = std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .map(|mut s| {
                s.validate();
                s
            })
            .unwrap_or_default();
        Self { path, current }
    }

    pub fn get(&self) -> Settings {
        self.current.clone()
    }

    pub fn replace(&mut self, mut next: Settings) -> Result<Settings, String> {
        next.validate();
        self.current = next.clone();
        self.persist(&next)?;
        Ok(next)
    }

    pub fn update(&mut self, patch: impl FnOnce(&mut Settings)) -> Result<Settings, String> {
        let mut next = self.current.clone();
        patch(&mut next);
        self.replace(next)
    }

    fn persist(&self, settings: &Settings) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &self.path).map_err(|e| e.to_string())
    }
}
