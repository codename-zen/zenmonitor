//! System information collector

use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub ip_address: Option<String>,
}

/// Collect static system information (called once at startup)
pub fn collect_system_info() -> SystemInfo {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let os = format!(
        "{} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "".to_string()),
    );

    let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());

    // Try to get the primary IP address
    let ip_address = get_primary_ip();

    SystemInfo {
        hostname,
        os,
        kernel,
        ip_address,
    }
}

/// Get system uptime in seconds
pub fn get_uptime() -> u64 {
    System::uptime()
}

/// Attempt to determine the primary IP address
fn get_primary_ip() -> Option<String> {
    // Read from /proc/net or use a UDP socket trick to find the outbound IP
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}
