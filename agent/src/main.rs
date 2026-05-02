//! ZenMonitor Agent - Main entry point
//!
//! Collects system metrics (CPU, RAM, Disk, Network, Processes) and reports
//! them to the ZenMonitor server at regular intervals.

mod config;
mod collectors;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "zenmonitor_agent=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting ZenMonitor Agent...");

    // Load configuration
    let config = config::AgentConfig::load().unwrap_or_default();
    tracing::info!("Agent ID: {}", config.agent_id);
    tracing::info!("Server URL: {}", config.server_url);
    tracing::info!("Report interval: {}s", config.report_interval);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Collect system info once
    let system_info = collectors::system::collect_system_info();
    tracing::info!("Hostname: {}", system_info.hostname);

    // Main reporting loop
    loop {
        match collectors::collect_full_report(&config, &system_info).await {
            Ok(report) => {
                let url = format!("{}/api/agents/report", config.server_url);
                match client.post(&url)
                    .header("X-API-Key", &config.api_key)
                    .json(&report)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            tracing::debug!("Report sent successfully");
                        } else {
                            tracing::warn!("Server returned {}: {:?}", resp.status(), resp.text().await);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to send report: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to collect metrics: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(config.report_interval)).await;
    }
}
