//! Frame statistics.
//!
//! Definitions used everywhere in the app:
//! * Average FPS          = frames / elapsed seconds, where elapsed is the sum of frame intervals.
//! * Median FPS           = 1000 / median frame time.
//! * 1% low / 0.1% low    = 1000 / (mean of the slowest 1% / 0.1% of frame times).
//! * Percentiles          = nearest-rank on the ascending frame-time list.
//! * Stutter frame        = frame time > max(spike_multiplier * median, spike_floor_ms).
//!
//! Lows are only reported once enough frames exist for the tail to be meaningful.

use crate::types::{FrameStats, HistogramBin};

pub const MIN_FRAMES_P1: usize = 300;
pub const MIN_FRAMES_P01: usize = 1000;

fn sort_asc(frames: &[f64]) -> Vec<f64> {
    let mut sorted: Vec<f64> = frames.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted
}

/// Nearest-rank percentile over an ascending list.
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let clamped = p.clamp(0.0, 100.0);
    let rank = ((clamped / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted.get(rank.min(sorted.len() - 1)).copied()
}

/// Mean of the slowest `fraction` of frame times, expressed as FPS.
pub fn low_fps(frames: &[f64], fraction: f64) -> Option<f64> {
    if frames.len() < 10 || fraction <= 0.0 {
        return None;
    }
    let mut sorted: Vec<f64> = frames.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let take = ((sorted.len() as f64) * fraction).ceil() as usize;
    let take = take.clamp(1, sorted.len());
    let mean = sorted[..take].iter().sum::<f64>() / take as f64;
    (mean > 0.0).then(|| 1000.0 / mean)
}

pub fn analyze(frames: &[f64], spike_multiplier: f64, spike_floor_ms: f64) -> FrameStats {
    let valid: Vec<f64> = frames.iter().copied().filter(|v| v.is_finite() && *v > 0.0).collect();
    if valid.is_empty() {
        return FrameStats { insufficient_samples: true, ..Default::default() };
    }
    let sorted = sort_asc(&valid);
    let count = valid.len();
    let total: f64 = valid.iter().sum();
    let mean = total / count as f64;
    let median = percentile(&sorted, 50.0).unwrap_or(mean);
    let variance = valid.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / count as f64;
    let cv = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };
    let threshold = (spike_multiplier * median).max(spike_floor_ms);
    let spikes: Vec<f64> = valid.iter().copied().filter(|f| *f > threshold).collect();

    FrameStats {
        frames: count as i64,
        duration_ms: total,
        avg_fps: Some(1000.0 / mean),
        median_fps: Some(1000.0 / median),
        p1_low: if count >= MIN_FRAMES_P1 { low_fps(&valid, 0.01) } else { None },
        p01_low: if count >= MIN_FRAMES_P01 { low_fps(&valid, 0.001) } else { None },
        avg_frame_ms: Some(mean),
        median_frame_ms: Some(median),
        p95_frame_ms: percentile(&sorted, 95.0),
        p99_frame_ms: percentile(&sorted, 99.0),
        min_frame_ms: sorted.first().copied(),
        max_frame_ms: sorted.last().copied(),
        cv: Some(cv),
        stutter_count: spikes.len() as i64,
        stutter_ms_total: spikes.iter().map(|f| f - threshold).sum(),
        insufficient_samples: count < MIN_FRAMES_P1,
    }
}

/// Linear histogram of frame times, clipped to the 99.5th percentile so a single
/// outlier cannot flatten the distribution.
pub fn histogram(frames: &[f64], bins: usize) -> Vec<HistogramBin> {
    let sorted = sort_asc(frames);
    if sorted.is_empty() || bins == 0 {
        return Vec::new();
    }
    let low = sorted[0];
    let high = percentile(&sorted, 99.5).unwrap_or(low).max(low + 0.5);
    let width = (high - low) / bins as f64;
    let mut counts = vec![0u32; bins];
    for value in &sorted {
        if *value < low || *value > high {
            continue;
        }
        let index = (((value - low) / width).floor() as usize).min(bins - 1);
        counts[index] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(index, count)| HistogramBin {
            from_ms: low + width * index as f64,
            to_ms: low + width * (index + 1) as f64,
            count,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steady(n: usize, ms: f64) -> Vec<f64> {
        vec![ms; n]
    }

    #[test]
    fn steady_60fps_is_exact() {
        let stats = analyze(&steady(600, 16.6667), 2.0, 33.3);
        assert!((stats.avg_fps.unwrap() - 60.0).abs() < 0.01);
        assert!((stats.median_fps.unwrap() - 60.0).abs() < 0.01);
        assert_eq!(stats.stutter_count, 0);
        assert!(!stats.insufficient_samples);
    }

    #[test]
    fn average_uses_elapsed_time_not_map() {
        // 99 frames at 10ms and 1 frame at 1000ms: mean 19.9ms -> 50.25 fps.
        let mut frames = steady(99, 10.0);
        frames.push(1000.0);
        let stats = analyze(&frames, 2.0, 33.3);
        let expected = 100.0 * 1000.0 / (99.0 * 10.0 + 1000.0);
        assert!((stats.avg_fps.unwrap() - expected).abs() < 0.001);
    }

    #[test]
    fn one_percent_low_uses_slowest_frames() {
        let mut frames = steady(990, 10.0);
        for _ in 0..10 {
            frames.push(100.0);
        }
        let stats = analyze(&frames, 2.0, 33.3);
        // slowest 1% of 1000 frames = 10 frames at 100ms -> 10 fps
        assert!((stats.p1_low.unwrap() - 10.0).abs() < 0.01);
        assert_eq!(stats.stutter_count, 10);
    }

    #[test]
    fn lows_require_enough_samples() {
        let stats = analyze(&steady(120, 16.0), 2.0, 33.3);
        assert!(stats.p1_low.is_none());
        assert!(stats.p01_low.is_none());
        assert!(stats.insufficient_samples);
        let stats = analyze(&steady(1500, 16.0), 2.0, 33.3);
        assert!(stats.p1_low.is_some());
        assert!(stats.p01_low.is_some());
    }

    #[test]
    fn percentile_is_nearest_rank() {
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&sorted, 0.0), Some(1.0));
        assert_eq!(percentile(&sorted, 50.0), Some(3.0));
        assert_eq!(percentile(&sorted, 100.0), Some(5.0));
        assert_eq!(percentile(&[], 50.0), None);
    }

    #[test]
    fn empty_input_is_flagged_not_zeroed() {
        let stats = analyze(&[], 2.0, 33.3);
        assert!(stats.insufficient_samples);
        assert!(stats.avg_fps.is_none());
        assert_eq!(stats.frames, 0);
    }

    #[test]
    fn histogram_counts_every_frame_once() {
        let frames = steady(100, 20.0);
        let bins = histogram(&frames, 10);
        let total: u32 = bins.iter().map(|b| b.count).sum();
        assert_eq!(total, 100);
    }
}
