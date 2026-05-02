//! TCP port monitor
//!
//! Checks if a TCP port is open and measures connection time.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use crate::models::*;
use crate::sse::SseEvent;
use crate::AppState;

/// Run the TCP port monitoring loop
pub async fn run(state: Arc<AppState>) {
    let interval = Duration::from_secs(state.config.tcp_check_interval);

    loop {
        let monitors = match state.db.get_monitors() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to fetch monitors: {}", e);
                tokio::time::sleep(interval).await;
                continue;
            }
        };

        for monitor in monitors.iter().filter(|m| {
            m.enabled && matches!(m.monitor_type, MonitorType::Tcp)
        }) {
            let result = check_tcp(monitor).await;

            if let Err(e) = state.db.insert_check_result(&result) {
                tracing::error!("Failed to store TCP result: {}", e);
            }

            let _ = state.tx.send(SseEvent::CheckResult {
                monitor_id: result.monitor_id.clone(),
                status: result.status.as_str().to_string(),
                response_time_ms: result.response_time_ms,
                message: result.message.clone(),
            });
        }

        tokio::time::sleep(interval).await;
    }
}

/// Perform a single TCP port check
async fn check_tcp(monitor: &Monitor) -> CheckResult {
    let port = monitor.port.unwrap_or(80);
    let addr = format!("{}:{}", monitor.target, port);
    let timeout = Duration::from_secs(5);

    let start = Instant::now();

    match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => {
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            CheckResult {
                monitor_id: monitor.id.clone(),
                status: CheckStatus::Up,
                response_time_ms: Some(elapsed),
                status_code: None,
                message: Some(format!("TCP port {} open ({:.2}ms)", port, elapsed)),
                checked_at: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Ok(Err(e)) => CheckResult {
            monitor_id: monitor.id.clone(),
            status: CheckStatus::Down,
            response_time_ms: None,
            status_code: None,
            message: Some(format!("TCP connection failed: {}", e)),
            checked_at: Some(chrono::Utc::now().to_rfc3339()),
        },
        Err(_) => CheckResult {
            monitor_id: monitor.id.clone(),
            status: CheckStatus::Down,
            response_time_ms: None,
            status_code: None,
            message: Some(format!("TCP connection timed out ({}s)", timeout.as_secs())),
            checked_at: Some(chrono::Utc::now().to_rfc3339()),
        },
    }
}
