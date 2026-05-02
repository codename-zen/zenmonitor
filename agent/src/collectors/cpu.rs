//! CPU usage collector

use sysinfo::System;
use std::time::Duration;

/// Collect overall CPU usage percentage (requires a brief delay for measurement)
pub async fn collect_cpu_usage() -> f64 {
    let mut sys = System::new();
    sys.refresh_cpu_usage();

    // sysinfo needs a delay between refreshes for accurate CPU measurement
    tokio::time::sleep(Duration::from_millis(500)).await;
    sys.refresh_cpu_usage();

    let cpus = sys.cpus();
    if cpus.is_empty() {
        return 0.0;
    }

    let total: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    (total / cpus.len() as f32) as f64
}
