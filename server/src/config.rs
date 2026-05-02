//! Server configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to listen on (e.g., "0.0.0.0:3000")
    pub listen_addr: String,
    /// Path to SQLite database file
    pub database_path: String,
    /// Interval in seconds for HTTP/HTTPS checks
    pub http_check_interval: u64,
    /// Interval in seconds for ping checks
    pub ping_check_interval: u64,
    /// Interval in seconds for TCP port checks
    pub tcp_check_interval: u64,
    /// Interval in seconds for SSL certificate checks
    pub ssl_check_interval: u64,
    /// HTTP request timeout in seconds
    pub http_timeout: u64,
    /// Agent API key for authentication
    pub agent_api_key: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:3000".to_string(),
            database_path: "zenmonitor.db".to_string(),
            http_check_interval: 60,
            ping_check_interval: 30,
            tcp_check_interval: 60,
            ssl_check_interval: 3600,
            http_timeout: 10,
            agent_api_key: "change-me-in-production".to_string(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from `zenmonitor-server.toml` or use defaults
    pub fn load() -> Option<Self> {
        let path = std::env::var("ZENMONITOR_CONFIG")
            .unwrap_or_else(|_| "zenmonitor-server.toml".to_string());

        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }
}
