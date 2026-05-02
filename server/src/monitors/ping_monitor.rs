//! ICMP Ping monitor
//!
//! Performs ICMP echo requests to check host availability and latency.

use std::sync::Arc;
use std::time::Duration;
use std::net::IpAddr;
use crate::models::*;
use crate::sse::SseEvent;
use crate::AppState;

/// Run the ping monitoring loop
pub async fn run(state: Arc<AppState>) {
    let interval = Duration::from_secs(state.config.ping_check_interval);

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
            m.enabled && matches!(m.monitor_type, MonitorType::Ping)
        }) {
            let result = check_ping(monitor).await;

            if let Err(e) = state.db.insert_check_result(&result) {
                tracing::error!("Failed to store ping result: {}", e);
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

/// Perform a single ICMP ping check
async fn check_ping(monitor: &Monitor) -> CheckResult {
    // Resolve the target to an IP address
    let ip: IpAddr = match monitor.target.parse() {
        Ok(ip) => ip,
        Err(_) => {
            // Try DNS resolution
            match tokio::net::lookup_host(format!("{}:0", monitor.target)).await {
                Ok(mut addrs) => match addrs.next() {
                    Some(addr) => addr.ip(),
                    None => {
                        return CheckResult {
                            monitor_id: monitor.id.clone(),
                            status: CheckStatus::Down,
                            response_time_ms: None,
                            status_code: None,
                            message: Some("DNS resolution failed: no addresses".to_string()),
                            checked_at: Some(chrono::Utc::now().to_rfc3339()),
                        };
                    }
                },
                Err(e) => {
                    return CheckResult {
                        monitor_id: monitor.id.clone(),
                        status: CheckStatus::Down,
                        response_time_ms: None,
                        status_code: None,
                        message: Some(format!("DNS resolution failed: {}", e)),
                        checked_at: Some(chrono::Utc::now().to_rfc3339()),
                    };
                }
            }
        }
    };

    // Perform ICMP ping using surge-ping
    match surge_ping::ping(ip, &[0u8; 8]).await {
        Ok((_packet, duration)) => {
            let ms = duration.as_secs_f64() * 1000.0;
            CheckResult {
                monitor_id: monitor.id.clone(),
                status: CheckStatus::Up,
                response_time_ms: Some(ms),
                status_code: None,
                message: Some(format!("Ping OK: {:.2}ms", ms)),
                checked_at: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Err(e) => CheckResult {
            monitor_id: monitor.id.clone(),
            status: CheckStatus::Down,
            response_time_ms: None,
            status_code: None,
            message: Some(format!("Ping failed: {}", e)),
            checked_at: Some(chrono::Utc::now().to_rfc3339()),
        },
    }
}
