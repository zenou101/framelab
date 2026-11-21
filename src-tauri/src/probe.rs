//! Headless verification mode: prints one real snapshot as JSON so the
//! monitoring path can be checked without the GUI.

use std::time::Duration;

pub fn run() {
    let mut sampler = crate::sensors::Sampler::new();
    let hardware = crate::hardware::collect_hardware(&sampler);
    // PDH rate counters need a second sample before they report real values.
    std::thread::sleep(Duration::from_millis(1200));
    let snapshot = sampler.snapshot(1000);
    let sidecar = crate::fps::sidecar_path(None).map(|path| {
        serde_json::json!({
            "path": path.to_string_lossy(),
            "integrity": crate::fps::sidecar_integrity(&path),
        })
    });
    let payload = serde_json::json!({
        "probe": "framelab.sensors",
        "hardware": hardware,
        "snapshot": snapshot,
        "presentmon": sidecar,
        "frame_stats_selftest": crate::stats::analyze(&[16.6, 16.7, 16.6, 50.0], 2.0, 33.3),
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(text) => println!("{text}"),
        Err(error) => eprintln!("probe serialization failed: {error}"),
    }
}
