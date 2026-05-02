//! HTTP/HTTPS endpoint monitor
//!
//! Periodically checks HTTP/HTTPS endpoints for status code and response time.

use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::models::*;
use crate::sse::SseEvent;
use crate::AppState;

/// Run the HTTP/HTTPS monitoring loop
pub async fn run(state: Arc<AppState>) {
    let interval = Duration::from_secs(state.config.http_check_interval);
    let timeout = Duration::from_secs(state.config.http_timeout);

    let client = reqwest::Client::builder()
        .timeout(timeout)
        .danger_accept_invalid_certs(false)
        .build()
        .expect("Failed to build HTTP client");

    loop {
        // Get all HTTP/HTTPS monitors
        let monitors = match state.db.get_monitors() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to fetch monitors: {}", e);
                tokio::time::sleep(interval).await;
                continue;
            }
        };

        for monitor in monitors.iter().filter(|m| {
            m.enabled && matches!(m.monitor_type, MonitorType::Http | MonitorType::Https)
        }) {
            let result = check_http(&client, monitor).await;

            // Store result
            if let Err(e) = state.db.insert_check_result(&result) {
                tracing::error!("Failed to store check result: {}", e);
            }

            // Broadcast via SSE
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

/// Perform a single HTTP check
async fn check_http(client: &reqwest::Client, monitor: &Monitor) -> CheckResult {
    let url = &monitor.target;
    let start = Instant::now();

    match client.get(url).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            let status_code = response.status().as_u16();
            let status = if response.status().is_success() {
                CheckStatus::Up
            } else if response.status().is_server_error() {
                CheckStatus::Down
            } else {
                CheckStatus::Degraded
            };

            CheckResult {
                monitor_id: monitor.id.clone(),
                status,
                response_time_ms: Some(elapsed),
                status_code: Some(status_code),
                message: Some(format!("HTTP {}", status_code)),
                checked_at: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Err(e) => CheckResult {
            monitor_id: monitor.id.clone(),
            status: CheckStatus::Down,
            response_time_ms: None,
            status_code: None,
            message: Some(format!("Error: {}", e)),
            checked_at: Some(chrono::Utc::now().to_rfc3339()),
        },
    }
}
