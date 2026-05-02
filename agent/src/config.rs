//! Agent configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier
    pub agent_id: String,
    /// ZenMonitor server URL (e.g., "http://monitor.example.com:3000")
    pub server_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Reporting interval in seconds
    pub report_interval: u64,
    /// Number of top processes to report
    pub top_processes: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: uuid::Uuid::new_v4().to_string(),
            server_url: "http://localhost:3000".to_string(),
            api_key: "change-me-in-production".to_string(),
            report_interval: 30,
            top_processes: 10,
        }
    }
}

impl AgentConfig {
    /// Load configuration from `zenmonitor-agent.toml` or use defaults
    pub fn load() -> Option<Self> {
        let path = std::env::var("ZENMONITOR_AGENT_CONFIG")
            .unwrap_or_else(|_| "zenmonitor-agent.toml".to_string());

        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }
}
