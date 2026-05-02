//! SSL Certificate expiry monitor
//!
//! Connects to HTTPS endpoints and checks certificate validity and expiry dates.

use std::sync::Arc;
use std::time::Duration;
use std::net::TcpStream as StdTcpStream;
use std::io::Write;
use crate::models::*;
use crate::sse::SseEvent;
use crate::AppState;

/// Run the SSL certificate monitoring loop
pub async fn run(state: Arc<AppState>) {
    let interval = Duration::from_secs(state.config.ssl_check_interval);

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
            m.enabled && matches!(m.monitor_type, MonitorType::Ssl | MonitorType::Https)
        }) {
            let result = check_ssl(monitor).await;

            if let Err(e) = state.db.insert_check_result(&result) {
                tracing::error!("Failed to store SSL result: {}", e);
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

/// Perform SSL certificate check
async fn check_ssl(monitor: &Monitor) -> CheckResult {
    let host = extract_hostname(&monitor.target);
    let port = monitor.port.unwrap_or(443);

    // Run the blocking TLS check in a spawn_blocking context
    let host_clone = host.clone();
    let monitor_id = monitor.id.clone();

    match tokio::task::spawn_blocking(move || {
        check_ssl_blocking(&host_clone, port)
    }).await {
        Ok(Ok(info)) => {
            let status = if info.days_until_expiry > 30 {
                CheckStatus::Up
            } else if info.days_until_expiry > 7 {
                CheckStatus::Degraded
            } else {
                CheckStatus::Down
            };

            CheckResult {
                monitor_id,
                status,
                response_time_ms: None,
                status_code: None,
                message: Some(format!(
                    "SSL cert expires in {} days ({})",
                    info.days_until_expiry, info.not_after
                )),
                checked_at: Some(chrono::Utc::now().to_rfc3339()),
            }
        }
        Ok(Err(e)) => CheckResult {
            monitor_id,
            status: CheckStatus::Down,
            response_time_ms: None,
            status_code: None,
            message: Some(format!("SSL check failed: {}", e)),
            checked_at: Some(chrono::Utc::now().to_rfc3339()),
        },
        Err(e) => CheckResult {
            monitor_id,
            status: CheckStatus::Down,
            response_time_ms: None,
            status_code: None,
            message: Some(format!("Task error: {}", e)),
            checked_at: Some(chrono::Utc::now().to_rfc3339()),
        },
    }
}

/// Blocking SSL certificate check using rustls
fn check_ssl_blocking(host: &str, port: u16) -> Result<SslCertInfo, String> {
    use rustls::ClientConfig;
    use std::sync::Arc as StdArc;

    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name: rustls_pki_types::ServerName<'static> = host.to_string().try_into()
        .map_err(|e| format!("Invalid server name: {:?}", e))?;

    let mut conn = rustls::ClientConnection::new(StdArc::new(config), server_name)
        .map_err(|e| format!("TLS connection error: {}", e))?;

    let mut sock = StdTcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("TCP connection error: {}", e))?;

    sock.set_read_timeout(Some(Duration::from_secs(10))).ok();
    sock.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let mut tls = rustls::Stream::new(&mut conn, &mut sock);

    // Drive the handshake
    tls.write_all(b"").map_err(|e| format!("TLS handshake error: {}", e))?;

    // Get peer certificates
    let certs = conn.peer_certificates()
        .ok_or_else(|| "No peer certificates found".to_string())?;

    if certs.is_empty() {
        return Err("Empty certificate chain".to_string());
    }

    // Parse the first (leaf) certificate
    let cert_der = &certs[0].as_ref();
    let (_, cert) = x509_parser::parse_x509_certificate(cert_der)
        .map_err(|e| format!("Failed to parse certificate: {:?}", e))?;

    let not_before = cert.validity().not_before.to_rfc2822()
        .unwrap_or_else(|_| "unknown".to_string());
    let not_after = cert.validity().not_after.to_rfc2822()
        .unwrap_or_else(|_| "unknown".to_string());

    let now = chrono::Utc::now().timestamp();
    let expiry = cert.validity().not_after.timestamp();
    let days_until_expiry = (expiry - now) / 86400;

    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();

    Ok(SslCertInfo {
        monitor_id: String::new(), // filled by caller
        subject,
        issuer,
        not_before,
        not_after,
        days_until_expiry,
    })
}

/// Extract hostname from a URL or return as-is
fn extract_hostname(target: &str) -> String {
    if let Ok(url) = url::Url::parse(target) {
        url.host_str().unwrap_or(target).to_string()
    } else {
        target.to_string()
    }
}
