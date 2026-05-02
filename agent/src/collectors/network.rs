//! Network bandwidth collector

use serde::{Deserialize, Serialize};
use sysinfo::Networks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_rate_bps: f64,
    pub tx_rate_bps: f64,
}

/// Collect network interface statistics
pub fn collect_network_info() -> Vec<NetworkInfo> {
    let networks = Networks::new_with_refreshed_list();
    let mut result = Vec::new();

    for (name, data) in networks.list() {
        // Skip loopback
        if name == "lo" {
            continue;
        }

        result.push(NetworkInfo {
            interface: name.clone(),
            rx_bytes: data.total_received(),
            tx_bytes: data.total_transmitted(),
            // Rate calculation requires two samples; for now report cumulative
            // The server can compute rates from consecutive reports
            rx_rate_bps: data.received() as f64 * 8.0, // bits per refresh interval
            tx_rate_bps: data.transmitted() as f64 * 8.0,
        });
    }

    result
}
