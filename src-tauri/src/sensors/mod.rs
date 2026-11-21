pub mod pdh;

use crate::types::Metric;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

pub const SRC_PDH: &str = "Windows performance counters (PDH)";

const DISK_MISSING: &str = "The PhysicalDisk counter set is not present on this system (Windows performance counters for disks are disabled or unavailable), so disk throughput cannot be measured here.";
const NET_MISSING: &str = "The Network Interface counter set is not present on this system, so network throughput cannot be measured here.";

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// `\GPU Engine(pid_1_luid_0x0_0x128AF_phys_0_eng_0_engtype_3D)\Utilization
/// Percentage` -> `pid_1_luid_0x0_0x128AF_phys_0_eng_0_engtype_3D`
pub fn instance_name(path: &str) -> &str {
    match (path.find('('), path.rfind(')')) {
        (Some(open), Some(close)) if close > open => &path[open + 1..close],
        _ => path,
    }
}

pub fn luid_of(instance: &str) -> Option<&str> {
    let start = instance.find("luid_")?;
    let rest = &instance[start..];
    let end = rest.find("_phys_").unwrap_or(rest.len());
    Some(&rest[..end])
}

pub fn engine_type(instance: &str) -> Option<&str> {
    let start = instance.find("engtype_")?;
    Some(&instance[start + "engtype_".len()..])
}

struct Counters {
    cpu: Option<isize>,
    cpu_freq: Option<isize>,
    per_core: Option<isize>,
    gpu_engine: Option<isize>,
    gpu_mem: Option<isize>,
    disk: Option<isize>,
    net: Option<isize>,
}

pub struct Sampler {
    query: Option<pdh::PdhQuery>,
    counters: Option<Counters>,
    pub sys: System,
    pub selected_luid: Option<String>,
    pub gpu_method: String,
    pub gpu_instances: usize,
    pub gpu_name: String,
    pub vram_total: Option<u64>,
    pub adapter_notes: Vec<String>,
    warm: bool,
}

impl Sampler {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let (query, counters) = match pdh::PdhQuery::open() {
            Some(q) => {
                let counters = Counters {
                    cpu: q.add(r"\Processor Information(_Total)\% Processor Utility"),
                    cpu_freq: q.add(r"\Processor Information(_Total)\% of Maximum Frequency"),
                    per_core: q.add(r"\Processor Information(*)\% Processor Utility"),
                    gpu_engine: q.add(r"\GPU Engine(*)\Utilization Percentage"),
                    gpu_mem: q.add(r"\GPU Adapter Memory(*)\Dedicated Usage"),
                    disk: q.add(r"\PhysicalDisk(_Total)\Disk Bytes/sec"),
                    net: q.add(r"\Network Interface(*)\Bytes Total/sec"),
                };
                (Some(q), Some(counters))
            }
            None => (None, None),
        };
        let mut sampler = Self {
            query,
            counters,
            sys,
            selected_luid: None,
            gpu_method: "not sampled yet".into(),
            gpu_instances: 0,
            gpu_name: String::new(),
            vram_total: None,
            adapter_notes: Vec::new(),
            warm: false,
        };
        sampler.pick_adapter();
        if let Some((name, memory)) = crate::hardware::primary_adapter() {
            sampler.vram_total = Some(memory);
            sampler.gpu_name = name;
        }
        sampler
    }

    /// Choose the discrete adapter once, by the adapter with the largest
    /// dedicated allocation currently reported. Virtual/basic adapters report
    /// zero, so they lose this comparison deterministically.
    fn pick_adapter(&mut self) {
        let Some(mem) = self.counters.as_ref().and_then(|c| c.gpu_mem) else {
            self.adapter_notes.push("GPU Adapter Memory counter unavailable".into());
            return;
        };
        if self.query.as_ref().map(|q| q.collect()).unwrap_or(false) {
            let rows = pdh::read_array(mem);
            let mut best: Option<(String, f64)> = None;
            for (name, value) in rows {
                let instance = instance_name(&name);
                let Some(luid) = luid_of(instance) else { continue };
                let better = best.as_ref().map(|(_, v)| value > *v).unwrap_or(true);
                if better {
                    best = Some((luid.to_string(), value));
                }
            }
            if let Some((luid, _)) = best {
                self.adapter_notes.push(format!("Selected adapter {luid} by largest dedicated usage"));
                self.selected_luid = Some(luid);
            }
        }
    }

    fn collect(&self) -> bool {
        self.query.as_ref().map(|q| q.collect()).unwrap_or(false)
    }

    pub fn refresh_system(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
    }

    pub fn cpu_metric(&self) -> Metric {
        if let Some(counter) = self.counters.as_ref().and_then(|c| c.cpu) {
            if let Some(value) = pdh::read_value(counter) {
                return Metric::ok(format!("{SRC_PDH}: Processor Information"), value.clamp(0.0, 100.0));
            }
        }
        let value = self.sys.global_cpu_usage() as f64;
        if value.is_finite() && value >= 0.0 {
            Metric::ok("sysinfo (fallback)", value.clamp(0.0, 100.0))
        } else {
            Metric::unavailable(SRC_PDH, "CPU utilization counter returned no data")
        }
    }

    pub fn cpu_clock_metric(&self) -> Metric {
        if let Some(counter) = self.counters.as_ref().and_then(|c| c.cpu_freq) {
            if let Some(percent) = pdh::read_value(counter) {
                let base = self.sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
                if base > 0 {
                    return Metric::ok_method(
                        format!("{SRC_PDH}: % of Maximum Frequency"),
                        (percent / 100.0) * base as f64,
                        "scaled by base clock",
                    );
                }
            }
        }
        self.sys
            .cpus()
            .first()
            .map(|c| Metric::ok("sysinfo", c.frequency() as f64))
            .unwrap_or_else(|| Metric::unavailable("sysinfo", "CPU clock not reported"))
    }

    pub fn per_core(&self) -> Vec<f64> {
        if let Some(counter) = self.counters.as_ref().and_then(|c| c.per_core) {
            let rows = pdh::read_array(counter);
            let mut cores: Vec<f64> = rows
                .into_iter()
                .filter(|(name, _)| !instance_name(name).contains("_Total"))
                .map(|(_, value)| value.clamp(0.0, 100.0))
                .collect();
            if !cores.is_empty() {
                cores.truncate(64);
                return cores;
            }
        }
        self.sys.cpus().iter().take(64).map(|c| c.cpu_usage() as f64).collect()
    }
}

impl Sampler {
    pub fn gpu_metric(&mut self) -> Metric {
        let Some(counter) = self.counters.as_ref().and_then(|c| c.gpu_engine) else {
            return Metric::unavailable(SRC_PDH, "The GPU Engine counter set is not exposed on this system");
        };
        let rows = pdh::read_array(counter);
        if rows.is_empty() {
            return Metric::unavailable(SRC_PDH, "No GPU engine instances reported (no active graphics workload)");
        }
        let luid = self.selected_luid.clone();
        let mut values: Vec<f64> = Vec::new();
        for (name, value) in rows.iter() {
            let instance = instance_name(name);
            if let Some(l) = &luid {
                if luid_of(instance) != Some(l.as_str()) {
                    continue;
                }
            }
            if matches!(engine_type(instance), Some("3D") | Some("High Priority 3D")) {
                values.push(*value);
            }
        }
        self.gpu_instances = values.len();
        if values.is_empty() {
            self.gpu_method = "no 3D engine instances".into();
            return Metric::unavailable(SRC_PDH, "The selected adapter reported no 3D engine instances");
        }
        let sum: f64 = values.iter().sum();
        let (value, method) = if sum <= 100.5 {
            (sum.clamp(0.0, 100.0), format!("sum of {} validated 3D engine instances", values.len()))
        } else {
            (
                values.iter().copied().fold(f64::MIN, f64::max).clamp(0.0, 100.0),
                format!("busiest of {} instances (sum {}% exceeded 100%)", values.len(), sum.round()),
            )
        };
        self.gpu_method = method.clone();
        Metric::ok_method(SRC_PDH, value, method)
    }

    pub fn vram_used(&self) -> Metric {
        let Some(counter) = self.counters.as_ref().and_then(|c| c.gpu_mem) else {
            return Metric::unavailable(SRC_PDH, "GPU Adapter Memory counter unavailable");
        };
        let rows = pdh::read_array(counter);
        let target = self.selected_luid.clone();
        let found = rows.iter().find(|(name, _)| {
            let instance = instance_name(name);
            target.as_deref().map_or(true, |t| luid_of(instance) == Some(t))
        });
        match found {
            Some((_, value)) if *value >= 0.0 => Metric::ok(SRC_PDH, *value),
            _ => Metric::unavailable(SRC_PDH, "No dedicated memory reading for the selected adapter"),
        }
    }

    pub fn disk_metric(&self) -> Metric {
        match self.counters.as_ref().and_then(|c| c.disk).and_then(pdh::read_value) {
            Some(value) if value >= 0.0 => Metric::ok(SRC_PDH, value),
            _ => Metric::unavailable(SRC_PDH, DISK_MISSING),
        }
    }

    pub fn net_metric(&self) -> Metric {
        let Some(counter) = self.counters.as_ref().and_then(|c| c.net) else {
            return Metric::unavailable(SRC_PDH, NET_MISSING);
        };
        let rows = pdh::read_array(counter);
        if rows.is_empty() {
            return Metric::unavailable(SRC_PDH, "No active network interfaces");
        }
        let total: f64 = rows.iter().map(|(_, v)| *v).filter(|v| v.is_finite()).sum();
        Metric::ok_method(SRC_PDH, total, format!("sum of {} interfaces", rows.len()))
    }

    pub fn snapshot(&mut self, sampling_ms: u64) -> crate::types::Snapshot {
        let collected = self.collect();
        self.refresh_system();
        let cpu = self.cpu_metric();
        let clock = self.cpu_clock_metric();
        let gpu = self.gpu_metric();
        let per_core = self.per_core();
        let ram_total = self.sys.total_memory() as f64;
        let ram_used = self.sys.used_memory() as f64;
        let ram_percent = if ram_total > 0.0 { ram_used / ram_total * 100.0 } else { 0.0 };
        let vram_used = self.vram_used();
        let disk = self.disk_metric();
        let net = self.net_metric();
        let thermal_reason = "No supported die-temperature sensor. FrameLab does not present ACPI zone readings as CPU temperature; enable the LibreHardwareMonitor bridge in Settings to expose real sensors.";
        let _ = collected;
        if !self.warm {
            self.warm = true;
        }
        crate::types::Snapshot {
            ts_ms: now_ms(),
            sampling_ms,
            cpu_percent: cpu,
            cpu_clock_mhz: clock,
            cpu_per_core: per_core,
            ram_percent: Metric::ok("sysinfo", ram_percent.clamp(0.0, 100.0)),
            ram_used_bytes: Metric::ok("sysinfo", ram_used.max(0.0)),
            ram_total_bytes: Metric::ok("sysinfo", ram_total.max(0.0)),
            gpu_percent: gpu,
            vram_used_bytes: vram_used,
            vram_total_bytes: match self.vram_total {
                Some(total) => Metric::ok("Adapter registry (HardwareInformation.qwMemorySize)", total as f64),
                None => Metric::unavailable("Adapter registry", "Dedicated video memory not reported"),
            },
            disk_bytes_per_sec: disk,
            net_bytes_per_sec: net,
            cpu_temp_c: Metric::unavailable("thermal sensors", thermal_reason),
            gpu_temp_c: Metric::unavailable("thermal sensors", thermal_reason),
            frame_source_ready: false,
        }
    }
}
