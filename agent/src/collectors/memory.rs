//! Memory collector
//!
//! Reads /proc/meminfo directly for accurate Linux memory stats.
//! This is critical for Proxmox/Linux where the distinction between
//! actual used, cached, and available memory matters.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: f64,
    pub used_mb: f64,
    pub cached_mb: f64,
    pub available_mb: f64,
    pub buffers_mb: f64,
    pub swap_total_mb: f64,
    pub swap_used_mb: f64,
}

/// Collect memory information from /proc/meminfo for accurate Linux stats
pub fn collect_memory_info() -> MemoryInfo {
    // Try reading /proc/meminfo directly for accurate Linux stats
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        return parse_proc_meminfo(&content);
    }

    // Fallback to sysinfo
    collect_memory_sysinfo()
}

/// Parse /proc/meminfo for accurate memory breakdown
/// This correctly handles the Proxmox/Linux memory model:
/// - MemTotal: total physical RAM
/// - MemAvailable: memory available for new allocations (kernel estimate)
/// - Cached: page cache (can be reclaimed)
/// - Buffers: buffer cache (can be reclaimed)
/// - Actual used = Total - Available (NOT Total - Free)
fn parse_proc_meminfo(content: &str) -> MemoryInfo {
    let mut total_kb: u64 = 0;
    let mut free_kb: u64 = 0;
    let mut available_kb: u64 = 0;
    let mut buffers_kb: u64 = 0;
    let mut cached_kb: u64 = 0;
    let mut swap_total_kb: u64 = 0;
    let mut swap_free_kb: u64 = 0;
    let mut slab_reclaimable_kb: u64 = 0;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }

        let value: u64 = parts[1].parse().unwrap_or(0);
        match parts[0] {
            "MemTotal:" => total_kb = value,
            "MemFree:" => free_kb = value,
            "MemAvailable:" => available_kb = value,
            "Buffers:" => buffers_kb = value,
            "Cached:" => cached_kb = value,
            "SwapTotal:" => swap_total_kb = value,
            "SwapFree:" => swap_free_kb = value,
            "SReclaimable:" => slab_reclaimable_kb = value,
            _ => {}
        }
    }

    // If MemAvailable is not present (very old kernels), estimate it
    if available_kb == 0 {
        available_kb = free_kb + buffers_kb + cached_kb + slab_reclaimable_kb;
    }

    let total_mb = total_kb as f64 / 1024.0;
    let available_mb = available_kb as f64 / 1024.0;
    let cached_mb = (cached_kb + slab_reclaimable_kb) as f64 / 1024.0;
    let buffers_mb = buffers_kb as f64 / 1024.0;
    // Actual used = Total - Available (this is the correct metric for Proxmox/Linux)
    let used_mb = total_mb - available_mb;

    MemoryInfo {
        total_mb,
        used_mb,
        cached_mb,
        available_mb,
        buffers_mb,
        swap_total_mb: swap_total_kb as f64 / 1024.0,
        swap_used_mb: (swap_total_kb - swap_free_kb) as f64 / 1024.0,
    }
}

/// Fallback: collect memory info using sysinfo crate
fn collect_memory_sysinfo() -> MemoryInfo {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();

    let total_mb = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let available_mb = sys.available_memory() as f64 / 1024.0 / 1024.0;
    let used_mb = sys.used_memory() as f64 / 1024.0 / 1024.0;

    MemoryInfo {
        total_mb,
        used_mb,
        cached_mb: 0.0, // sysinfo doesn't expose this directly
        available_mb,
        buffers_mb: 0.0,
        swap_total_mb: sys.total_swap() as f64 / 1024.0 / 1024.0,
        swap_used_mb: sys.used_swap() as f64 / 1024.0 / 1024.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_meminfo() {
        let sample = r#"
MemTotal:       16384000 kB
MemFree:         2048000 kB
MemAvailable:    8192000 kB
Buffers:          512000 kB
Cached:          4096000 kB
SwapTotal:       2048000 kB
SwapFree:        1024000 kB
SReclaimable:     256000 kB
"#;
        let info = parse_proc_meminfo(sample);
        assert!((info.total_mb - 16000.0).abs() < 1.0);
        assert!((info.available_mb - 8000.0).abs() < 1.0);
        assert!((info.used_mb - 8000.0).abs() < 1.0);
        assert!(info.cached_mb > 0.0);
    }
}
