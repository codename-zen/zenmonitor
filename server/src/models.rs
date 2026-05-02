//! Data models for ZenMonitor

use serde::{Deserialize, Serialize};

// ─── Monitor Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorType {
    Http,
    Https,
    Ping,
    Tcp,
    Ssl,
}

impl MonitorType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Ping => "ping",
            Self::Tcp => "tcp",
            Self::Ssl => "ssl",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "http" => Self::Http,
            "https" => Self::Https,
            "ping" => Self::Ping,
            "tcp" => Self::Tcp,
            "ssl" => Self::Ssl,
            _ => Self::Http,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    Up,
    Down,
    Degraded,
    Unknown,
}

impl CheckStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Degraded => "degraded",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "up" => Self::Up,
            "down" => Self::Down,
            "degraded" => Self::Degraded,
            _ => Self::Unknown,
        }
    }
}

// ─── Monitor ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub monitor_type: MonitorType,
    pub target: String,
    pub port: Option<u16>,
    pub interval_seconds: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMonitorRequest {
    pub name: String,
    pub monitor_type: String,
    pub target: String,
    pub port: Option<u16>,
    pub interval_seconds: Option<u64>,
}

// ─── Check Result ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub monitor_id: String,
    pub status: CheckStatus,
    pub response_time_ms: Option<f64>,
    pub status_code: Option<u16>,
    pub message: Option<String>,
    pub checked_at: Option<String>,
}

// ─── SSL Certificate ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertInfo {
    pub monitor_id: String,
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub days_until_expiry: i64,
}

// ─── Agent Models ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub hostname: String,
    pub os: Option<String>,
    pub kernel: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: String,
    pub cpu_usage: f64,
    pub ram_total_mb: f64,
    pub ram_used_mb: f64,
    pub ram_cached_mb: f64,
    pub ram_available_mb: f64,
    pub uptime_seconds: u64,
    pub disks: Vec<DiskInfo>,
    pub network: Vec<NetworkInfo>,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_rate_bps: f64,
    pub tx_rate_bps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
}

/// Full agent report payload sent from agent to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReport {
    pub agent: AgentInfo,
    pub metrics: AgentMetrics,
}

// ─── API Responses ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}
