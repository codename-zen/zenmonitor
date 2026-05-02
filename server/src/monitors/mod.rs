//! Monitoring modules - background tasks that perform checks

pub mod http_monitor;
pub mod ping_monitor;
pub mod tcp_monitor;
pub mod ssl_monitor;

use std::sync::Arc;
use crate::AppState;

/// Start all monitoring background tasks
pub async fn start_all_monitors(state: Arc<AppState>) {
    tracing::info!("Starting background monitoring tasks...");

    // HTTP/HTTPS monitor
    let s = state.clone();
    tokio::spawn(async move {
        http_monitor::run(s).await;
    });

    // Ping (ICMP) monitor
    let s = state.clone();
    tokio::spawn(async move {
        ping_monitor::run(s).await;
    });

    // TCP port monitor
    let s = state.clone();
    tokio::spawn(async move {
        tcp_monitor::run(s).await;
    });

    // SSL certificate monitor
    let s = state.clone();
    tokio::spawn(async move {
        ssl_monitor::run(s).await;
    });

    tracing::info!("✅ All monitoring tasks started");
}
