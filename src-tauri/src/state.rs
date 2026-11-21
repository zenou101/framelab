use crate::bench::Recorder;
use crate::db::Db;
use crate::fps::Capture;
use crate::sensors::Sampler;
use crate::settings::SettingsStore;
use crate::types::{FrameTick, HardwareInfo, Snapshot};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub resource_dir: Option<PathBuf>,
}

pub struct AppState {
    pub sampler: Mutex<Sampler>,
    pub capture: Mutex<Capture>,
    pub recorder: Mutex<Recorder>,
    pub db: Mutex<Db>,
    pub settings: Mutex<SettingsStore>,
    pub hardware: Mutex<HardwareInfo>,
    pub last_snapshot: Mutex<Option<Snapshot>>,
    pub last_frame: Mutex<Option<FrameTick>>,
    pub paths: AppPaths,
    pub recovered: Mutex<Vec<i64>>,
    pub started_at_ms: i64,
}
