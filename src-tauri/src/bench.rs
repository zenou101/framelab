use crate::db::{Db, FinishedSession, NewSession};
use crate::fps::FrameBuffers;
use crate::stats;
use crate::types::{RecorderState, SampleSeries, SessionStats, Snapshot};

/// Hard cap on frames buffered between flushes. When it is reached recording
/// pauses and reports backpressure instead of growing without bound.
pub const MAX_PENDING: usize = 120_000;
const FLUSH_INTERVAL_MS: i64 = 10_000;
pub const SAMPLE_BYTES: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct Sample {
    pub ts_ms: i64,
    pub cpu: f32,
    pub gpu: f32,
    pub ram: f32,
    pub vram: f32,
    pub cpu_temp: f32,
    pub gpu_temp: f32,
}

impl Sample {
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        Self {
            ts_ms: snapshot.ts_ms,
            cpu: snapshot.cpu_percent.value.unwrap_or(f32::NAN as f64) as f32,
            gpu: snapshot.gpu_percent.value.unwrap_or(f32::NAN as f64) as f32,
            ram: snapshot.ram_percent.value.unwrap_or(f32::NAN as f64) as f32,
            vram: match (snapshot.vram_used_bytes.value, snapshot.vram_total_bytes.value) {
                (Some(used), Some(total)) if total > 0.0 => (used / total * 100.0) as f32,
                _ => f32::NAN,
            },
            cpu_temp: snapshot.cpu_temp_c.value.unwrap_or(f32::NAN as f64) as f32,
            gpu_temp: snapshot.gpu_temp_c.value.unwrap_or(f32::NAN as f64) as f32,
        }
    }

    pub fn to_bytes(&self) -> [u8; SAMPLE_BYTES] {
        let mut out = [0u8; SAMPLE_BYTES];
        out[0..8].copy_from_slice(&self.ts_ms.to_le_bytes());
        out[8..12].copy_from_slice(&self.cpu.to_le_bytes());
        out[12..16].copy_from_slice(&self.gpu.to_le_bytes());
        out[16..20].copy_from_slice(&self.ram.to_le_bytes());
        out[20..24].copy_from_slice(&self.vram.to_le_bytes());
        out[24..28].copy_from_slice(&self.cpu_temp.to_le_bytes());
        out[28..32].copy_from_slice(&self.gpu_temp.to_le_bytes());
        out
    }
}

pub fn decode_samples(bytes: &[u8]) -> SampleSeries {
    let mut series = SampleSeries::default();
    for record in bytes.chunks_exact(SAMPLE_BYTES) {
        let read_f32 = |start: usize| f32::from_le_bytes([record[start], record[start + 1], record[start + 2], record[start + 3]]);
        let ts = i64::from_le_bytes(record[0..8].try_into().unwrap_or([0; 8]));
        series.t_ms.push(ts);
        series.cpu.push(read_f32(8));
        series.gpu.push(read_f32(12));
        series.ram.push(read_f32(16));
        series.vram.push(read_f32(20));
        series.cpu_temp.push(read_f32(24));
        series.gpu_temp.push(read_f32(28));
    }
    series
}

#[derive(Default)]
pub struct Recorder {
    state: RecorderState,
    pending_displayed: Vec<f64>,
    pending_presented: Vec<f64>,
    pending_samples: Vec<Sample>,
    seen_displayed: u64,
    seen_presented: u64,
    chunk: i64,
    last_flush_ms: i64,
    paused: bool,
}

impl Recorder {
    pub fn state(&self) -> RecorderState {
        self.state.clone()
    }

    pub fn start(&mut self, db: &Db, game_id: Option<i64>, game_name: &str, process: &str, pid: u32, primary_source: &str, hardware_json: &str) -> Result<i64, String> {
        let started = crate::sensors::now_ms();
        let meta = serde_json::json!({ "process": process, "pid": pid }).to_string();
        let id = db.create_session(&NewSession {
            game_id,
            game_name: game_name.to_string(),
            started_at_ms: started,
            primary_source: primary_source.to_string(),
            hardware_json: hardware_json.to_string(),
            meta_json: meta,
        })?;
        *self = Self {
            state: RecorderState {
                active: true,
                session_id: Some(id),
                pid: Some(pid),
                process: Some(process.to_string()),
                started_at_ms: Some(started),
                ..Default::default()
            },
            chunk: 0,
            last_flush_ms: started,
            ..Default::default()
        };
        Ok(id)
    }

    pub fn ingest(&mut self, buffers: &FrameBuffers) {
        if !self.state.active || self.paused {
            return;
        }
        self.state.displayed_frames = buffers.captured_displayed;
        self.state.presented_frames = buffers.captured_presented;
        if buffers.captured_displayed > self.seen_displayed {
            let series = buffers.series(true);
            let delta = ((buffers.captured_displayed - self.seen_displayed) as usize).min(series.len());
            let start = series.len() - delta;
            self.pending_displayed.extend_from_slice(&series[start..]);
            self.seen_displayed = buffers.captured_displayed;
        }
        if buffers.captured_presented > self.seen_presented {
            let series = buffers.series(false);
            let delta = ((buffers.captured_presented - self.seen_presented) as usize).min(series.len());
            let start = series.len() - delta;
            self.pending_presented.extend_from_slice(&series[start..]);
            self.seen_presented = buffers.captured_presented;
        }
        if self.pending_displayed.len() >= MAX_PENDING || self.pending_presented.len() >= MAX_PENDING {
            self.paused = true;
            self.state.backpressure += 1;
            self.state.message = Some(
                "Recording paused: the write queue reached its 120,000-frame budget. Storage is not keeping up."
                    .into(),
            );
        }
    }

    pub fn push_sample(&mut self, sample: Sample) {
        if !self.state.active || self.paused {
            return;
        }
        if self.pending_samples.len() >= 7200 {
            self.pending_samples.remove(0);
        }
        self.pending_samples.push(sample);
    }

    pub fn maybe_flush(&mut self, db: &Db) {
        if !self.state.active {
            return;
        }
        let now = crate::sensors::now_ms();
        if now - self.last_flush_ms < FLUSH_INTERVAL_MS {
            return;
        }
        if let Some(id) = self.state.session_id {
            if self.flush(db, id).is_ok() {
                self.state.persisted_frames += (self.pending_displayed.len() + self.pending_presented.len()) as u64;
                self.pending_displayed.clear();
                self.pending_presented.clear();
                self.paused = false;
                self.state.message = None;
                self.last_flush_ms = now;
                self.chunk += 1;
            } else {
                self.state.message = Some("Recording paused: the database rejected a write.".into());
                self.paused = true;
            }
        }
    }

    fn flush(&mut self, db: &Db, id: i64) -> Result<(), String> {
        if !self.pending_displayed.is_empty() {
            db.append_frames(id, "displayed", self.chunk, self.last_flush_ms, &self.pending_displayed)?;
        }
        if !self.pending_presented.is_empty() {
            db.append_frames(id, "presented", self.chunk, self.last_flush_ms, &self.pending_presented)?;
        }
        if !self.pending_samples.is_empty() {
            let mut bytes = Vec::with_capacity(self.pending_samples.len() * SAMPLE_BYTES);
            for sample in &self.pending_samples {
                bytes.extend_from_slice(&sample.to_bytes());
            }
            db.append_samples(id, self.chunk, self.last_flush_ms, &bytes)?;
            self.pending_samples.clear();
        }
        Ok(())
    }

    pub fn stop(&mut self, db: &Db, status: &str) -> Option<(i64, SessionStats)> {
        let id = self.state.session_id?;
        let _ = self.flush(db, id);
        let displayed = db.load_frames(id, "displayed").unwrap_or_default();
        let presented = db.load_frames(id, "presented").unwrap_or_default();
        let displayed_stats = stats::analyze(&displayed, 2.0, 33.3);
        let presented_stats = stats::analyze(&presented, 2.0, 33.3);
        let ended = crate::sensors::now_ms();
        let duration = ended - self.state.started_at_ms.unwrap_or(ended);
        let result = db.finish_session(
            id,
            &FinishedSession {
                ended_at_ms: ended,
                duration_ms: duration,
                status: status.into(),
                recovered: false,
                frames_displayed: Some(displayed_stats.frames),
                frames_presented: Some(presented_stats.frames),
                avg_fps_displayed: displayed_stats.avg_fps,
                avg_fps_presented: presented_stats.avg_fps,
                median_fps_displayed: displayed_stats.median_fps,
                p1_low_displayed: displayed_stats.p1_low,
                p01_low_displayed: displayed_stats.p01_low,
                p1_low_presented: presented_stats.p1_low,
                stutter_count: Some(displayed_stats.stutter_count),
            },
        );
        self.state.active = false;
        self.state.session_id = None;
        if result.is_err() {
            return None;
        }
        Some((id, SessionStats { displayed: displayed_stats, presented: presented_stats }))
    }
}
