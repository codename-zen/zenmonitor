//! Top processes collector

use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
}

/// Collect top N processes sorted by CPU usage
pub fn collect_top_processes(top_n: usize) -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut procs: Vec<ProcessInfo> = sys.processes().iter().map(|(pid, proc_)| {
        ProcessInfo {
            pid: pid.as_u32(),
            name: proc_.name().to_string(),
            cpu_percent: proc_.cpu_usage() as f64,
            memory_mb: proc_.memory() as f64 / 1024.0 / 1024.0,
        }
    }).collect();

    // Sort by CPU usage descending
    procs.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(top_n);
    procs
}
