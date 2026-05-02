//! Disk usage collector

use serde::{Deserialize, Serialize};
use sysinfo::Disks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
}

/// Collect disk usage for all mounted filesystems
pub fn collect_disk_info() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut result = Vec::new();

    for disk in disks.list() {
        let mount = disk.mount_point().to_string_lossy().to_string();

        // Skip pseudo-filesystems
        if mount.starts_with("/sys") || mount.starts_with("/proc")
            || mount.starts_with("/dev") || mount.starts_with("/run")
            || mount.starts_with("/snap")
        {
            continue;
        }

        let total = disk.total_space() as f64 / 1_073_741_824.0; // bytes to GB
        let available = disk.available_space() as f64 / 1_073_741_824.0;
        let used = total - available;
        let usage_percent = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };

        result.push(DiskInfo {
            mount_point: mount,
            total_gb: (total * 100.0).round() / 100.0,
            used_gb: (used * 100.0).round() / 100.0,
            available_gb: (available * 100.0).round() / 100.0,
            usage_percent: (usage_percent * 10.0).round() / 10.0,
        });
    }

    result
}
