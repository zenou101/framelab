//! Rule-based diagnostics.
//!
//! Every rule is a configurable heuristic over measurements this build actually
//! recorded. Findings carry the numbers that triggered them and are labelled
//! `confidence: measured | suspected`; nothing here claims a root cause.

use crate::types::{AnalysisReport, Finding, FrameStats, SampleSeries, Thresholds};

fn finding(code: &str, severity: &str, confidence: &str, title: &str, explanation: String, next_step: &str, evidence: Vec<String>) -> Finding {
    Finding {
        code: code.into(),
        severity: severity.into(),
        title: title.into(),
        explanation,
        next_step: next_step.into(),
        confidence: confidence.into(),
        evidence,
        heuristic: true,
    }
}

fn sample_at(samples: &SampleSeries, relative_ms: f64, pick: impl Fn(usize) -> f32) -> Option<f32> {
    if samples.t_ms.is_empty() {
        return None;
    }
    let base = samples.t_ms[0] as f64;
    let target = base + relative_ms;
    let mut best: Option<f32> = None;
    for index in 0..samples.t_ms.len() {
        let value = pick(index);
        if samples.t_ms[index] as f64 <= target {
            if value.is_finite() {
                best = Some(value);
            }
        } else {
            break;
        }
    }
    best.or_else(|| {
        let value = pick(0);
        value.is_finite().then_some(value)
    })
}

fn median_of(values: &[f32]) -> Option<f32> {
    let mut filtered: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if filtered.is_empty() {
        return None;
    }
    filtered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(filtered[filtered.len() / 2])
}

pub fn analyze_session(frames: &[f64], stats: &FrameStats, samples: &SampleSeries, thresholds: &Thresholds) -> AnalysisReport {
    let mut findings = Vec::new();
    let mut not_evaluated = Vec::new();

    if stats.frames < 300 || frames.len() < 300 {
        not_evaluated.push("Fewer than 300 frames were captured. Frame pacing, spike clustering and pressure checks need more samples.".into());
        return AnalysisReport { findings, not_evaluated, thresholds: thresholds.clone() };
    }

    let median = stats.median_frame_ms.unwrap_or(0.0);
    let cv = stats.cv.unwrap_or(0.0);
    let p99 = stats.p99_frame_ms.unwrap_or(median);
    let threshold = (thresholds.spike_multiplier * median).max(thresholds.spike_floor_ms);

    if median > 0.0 && cv > thresholds.pacing_cv && p99 / median > thresholds.pacing_p99_ratio {
        findings.push(finding(
            "pacing_instability",
            "notice",
            "measured",
            "Frame pacing is unstable",
            format!(
                "Frame intervals vary far more than a steady cadence would. The coefficient of variation is {cv:.2} and the 99th-percentile frame time is {:.1}× the median.",
                p99 / median
            ),
            "Repeat the same scene with a frame cap or V-Sync to see whether the variation follows the render load.",
            vec![
                format!("Coefficient of variation: {cv:.2} (threshold {:.2})", thresholds.pacing_cv),
                format!("p99 frame time {p99:.1} ms vs median {median:.1} ms (ratio {:.2}, threshold {:.2})", p99 / median, thresholds.pacing_p99_ratio),
            ],
        ));
    }

    let mut spike_offsets: Vec<f64> = Vec::new();
    let mut elapsed = 0.0f64;
    for frame in frames {
        elapsed += *frame;
        if *frame > threshold {
            spike_offsets.push(elapsed);
        }
    }

    if spike_offsets.len() >= thresholds.spike_cluster_min as usize {
        let mut clusters = 0;
        let mut index = 0;
        while index < spike_offsets.len() {
            let window_end = spike_offsets[index] + thresholds.spike_cluster_window_ms as f64;
            let in_window = spike_offsets[index..].iter().take_while(|offset| **offset <= window_end).count();
            if in_window >= 2 {
                clusters += 1;
            }
            index += in_window.max(1);
        }
        if clusters > 0 {
            findings.push(finding(
                "spike_clusters",
                "warning",
                "measured",
                "Repeated frame-time spikes",
                format!(
                    "{} frames exceeded {:.1} ms, and {} of them occurred in bursts of two or more within {} ms.",
                    spike_offsets.len(), threshold, clusters, thresholds.spike_cluster_window_ms
                ),
                "Watch the same scene while closing background applications one at a time to see whether the bursts move.",
                vec![
                    format!("Spike frames: {}", spike_offsets.len()),
                    format!("Spike threshold: {threshold:.1} ms = max({:.1} × median, {:.1} ms)", thresholds.spike_multiplier, thresholds.spike_floor_ms),
                    format!("Bursts: {clusters}"),
                ],
            ));
        }
    }

    if spike_offsets.is_empty() {
        findings.push(finding(
            "steady_pacing",
            "info",
            "measured",
            "No frame-time spikes above the configured threshold",
            format!("Every recorded frame stayed at or below {threshold:.1} ms."),
            "If the game still feels uneven, lower the spike threshold in Settings and compare again.",
            vec![format!("Spike threshold: {threshold:.1} ms")],
        ));
    }

    let ram_at_spikes: Vec<f32> = spike_offsets.iter().filter_map(|offset| sample_at(samples, *offset, |i| samples.ram[i])).collect();
    if let Some(peak) = ram_at_spikes.iter().copied().filter(|v| v.is_finite()).fold(None, |acc: Option<f32>, v| Some(acc.map_or(v, |a| a.max(v)))) {
        let over = ram_at_spikes.iter().filter(|v| **v >= thresholds.ram_pressure_pct as f32).count();
        if over >= 3 {
            findings.push(finding(
                "ram_pressure",
                "notice",
                "suspected",
                "High memory utilisation coincides with spikes",
                format!("{over} spike moments were sampled while system RAM use was at or above {:.0}%. This is a correlation in the recording, not a proven cause.", thresholds.ram_pressure_pct),
                "Close unnecessary background applications and repeat the benchmark to see whether the spikes change.",
                vec![format!("Peak RAM at spike moments: {peak:.0}%"), format!("Spike samples above threshold: {over}")],
            ));
        }
    }

    if let Some(vram_peak) = samples.vram.iter().copied().filter(|v| v.is_finite()).fold(None, |acc: Option<f32>, v| Some(acc.map_or(v, |a| a.max(v)))) {
        let over = spike_offsets
            .iter()
            .filter_map(|offset| sample_at(samples, *offset, |i| samples.vram[i]))
            .filter(|v| *v >= thresholds.vram_pressure_pct as f32)
            .count();
        if over >= 3 {
            findings.push(finding(
                "vram_pressure",
                "notice",
                "suspected",
                "Video memory was near capacity during spikes",
                format!("{over} spike moments were sampled with VRAM at or above {:.0}% (peak {vram_peak:.0}%). Texture streaming would be consistent with this, but it is not proven here.", thresholds.vram_pressure_pct),
                "Lower texture quality by one step and repeat the same route to compare spike counts.",
                vec![format!("Peak VRAM: {vram_peak:.0}%"), format!("Spike samples above threshold: {over}")],
            ));
        }
    }

    if let Some(gpu_median) = median_of(&samples.gpu) {
        let dips = spike_offsets
            .iter()
            .filter_map(|offset| sample_at(samples, *offset, |i| samples.gpu[i]))
            .filter(|value| *value < gpu_median - thresholds.gpu_dip_points as f32)
            .count();
        if dips >= 3 {
            findings.push(finding(
                "gpu_dips",
                "notice",
                "suspected",
                "GPU utilisation dropped around spikes",
                format!("{dips} spike moments were sampled more than {:.0} points below the session median GPU utilisation of {gpu_median:.0}%.", thresholds.gpu_dip_points),
                "Check whether the scene changes what is on screen at those moments; a render or submission gap looks like this too.",
                vec![format!("Median GPU utilisation: {gpu_median:.0}%"), format!("Spike moments far below median: {dips}")],
            ));
        }
        if gpu_median < thresholds.gpu_underutilized_pct as f32 {
            if let Some(cpu_median) = median_of(&samples.cpu) {
                if cpu_median >= 60.0 {
                    findings.push(finding(
                        "cpu_limited_suspected",
                        "notice",
                        "suspected",
                        "The GPU was under-utilised while CPU use was high",
                        format!("Median GPU utilisation was {gpu_median:.0}% while total CPU use was {cpu_median:.0}%. That pattern is consistent with a CPU-side limit, but this recording cannot prove it."),
                        "Raise resolution or render scale and repeat: if FPS barely moves, the limit is likely on the CPU side.",
                        vec![
                            format!("Median GPU: {gpu_median:.0}% (threshold {:.0}%)", thresholds.gpu_underutilized_pct),
                            format!("Median CPU: {cpu_median:.0}%"),
                        ],
                    ));
                }
            }
        }
    } else {
        not_evaluated.push("GPU-bound / CPU-bound checks: no GPU utilisation samples were recorded.".into());
    }

    if !samples.gpu_temp.iter().any(|v| v.is_finite()) {
        not_evaluated.push("Thermal throttling: no die-temperature sensor was available, so temperature limits were not evaluated.".into());
    }

    not_evaluated.push("Background process contention: this build samples system totals rather than per-process usage during a benchmark.".into());

    AnalysisReport { findings, not_evaluated, thresholds: thresholds.clone() }
}
