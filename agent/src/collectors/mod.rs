//! Metric collectors for the agent

pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod processes;
pub mod system;

use crate::config::AgentConfig;
use serde::{Deserialize, Serialize};

/// Agent info reported to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub hostname: String,
    pub os: Option<String>,
    pub kernel: Option<String>,
    pub ip_address: Option<String>,
}

/// Full metrics payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: String,
    pub cpu_usage: f64,
    pub ram_total_mb: f64,
    pub ram_used_mb: f64,
    pub ram_cached_mb: f64,
    pub ram_available_mb: f64,
    pub uptime_seconds: u64,
    pub disks: Vec<disk::DiskInfo>,
    pub network: Vec<network::NetworkInfo>,
    pub processes: Vec<processes::ProcessInfo>,
}

/// Full report sent to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReport {
    pub agent: AgentInfo,
    pub metrics: AgentMetrics,
}

/// Collect a full report with all metrics
pub async fn collect_full_report(
    config: &AgentConfig,
    sys_info: &system::SystemInfo,
) -> Result<AgentReport, Box<dyn std::error::Error>> {
    let cpu_usage = cpu::collect_cpu_usage().await;
    let mem = memory::collect_memory_info();
    let disks = disk::collect_disk_info();
    let network = network::collect_network_info();
    let procs = processes::collect_top_processes(config.top_processes);
    let uptime = system::get_uptime();

    let agent = AgentInfo {
        id: config.agent_id.clone(),
        hostname: sys_info.hostname.clone(),
        os: Some(sys_info.os.clone()),
        kernel: Some(sys_info.kernel.clone()),
        ip_address: sys_info.ip_address.clone(),
    };

    let metrics = AgentMetrics {
        agent_id: config.agent_id.clone(),
        cpu_usage,
        ram_total_mb: mem.total_mb,
        ram_used_mb: mem.used_mb,
        ram_cached_mb: mem.cached_mb,
        ram_available_mb: mem.available_mb,
        uptime_seconds: uptime,
        disks,
        network,
        processes: procs,
    };

    Ok(AgentReport { agent, metrics })
}
