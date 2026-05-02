//! API endpoint handlers

use axum::{
    extract::{Path, Query, State},
    response::sse::Sse,
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;

use crate::models::*;
use crate::sse::{self, SseEvent};
use crate::AppState;

/// List all monitors
pub async fn list_monitors(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<Monitor>>> {
    match state.db.get_monitors() {
        Ok(monitors) => Json(ApiResponse::ok(monitors)),
        Err(e) => Json(ApiResponse::err(format!("Database error: {}", e))),
    }
}

/// Create a new monitor
pub async fn create_monitor(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateMonitorRequest>,
) -> Json<ApiResponse<Monitor>> {
    let monitor = Monitor {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        monitor_type: MonitorType::from_str(&req.monitor_type),
        target: req.target,
        port: req.port,
        interval_seconds: req.interval_seconds.unwrap_or(60),
        enabled: true,
    };

    match state.db.insert_monitor(&monitor) {
        Ok(_) => {
            // Notify SSE subscribers
            let _ = state.tx.send(SseEvent::MonitorUpdate {
                action: "created".to_string(),
                monitor_id: monitor.id.clone(),
            });
            Json(ApiResponse::ok(monitor))
        }
        Err(e) => Json(ApiResponse::err(format!("Failed to create monitor: {}", e))),
    }
}

/// Delete a monitor
pub async fn delete_monitor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    match state.db.delete_monitor(&id) {
        Ok(true) => {
            let _ = state.tx.send(SseEvent::MonitorUpdate {
                action: "deleted".to_string(),
                monitor_id: id.clone(),
            });
            Json(ApiResponse::ok("Monitor deleted".to_string()))
        }
        Ok(false) => Json(ApiResponse::err("Monitor not found")),
        Err(e) => Json(ApiResponse::err(format!("Database error: {}", e))),
    }
}

/// Query parameters for results endpoint
#[derive(serde::Deserialize)]
pub struct ResultsQuery {
    pub limit: Option<u32>,
}

/// Get check results for a monitor
pub async fn get_monitor_results(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<ResultsQuery>,
) -> Json<ApiResponse<Vec<CheckResult>>> {
    let limit = params.limit.unwrap_or(50);
    match state.db.get_check_results(&id, limit) {
        Ok(results) => Json(ApiResponse::ok(results)),
        Err(e) => Json(ApiResponse::err(format!("Database error: {}", e))),
    }
}

/// List all agents
pub async fn list_agents(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<AgentInfo>>> {
    match state.db.get_agents() {
        Ok(agents) => Json(ApiResponse::ok(agents)),
        Err(e) => Json(ApiResponse::err(format!("Database error: {}", e))),
    }
}

/// Receive agent report
pub async fn agent_report(
    State(state): State<Arc<AppState>>,
    Json(report): Json<AgentReport>,
) -> Json<ApiResponse<String>> {
    // Upsert agent info
    if let Err(e) = state.db.upsert_agent(&report.agent) {
        return Json(ApiResponse::err(format!("Failed to upsert agent: {}", e)));
    }

    // Insert metrics
    match state.db.insert_agent_metrics(&report.metrics) {
        Ok(_metric_id) => {
            // Broadcast via SSE
            let _ = state.tx.send(SseEvent::AgentMetrics {
                agent_id: report.agent.id.clone(),
                hostname: report.agent.hostname.clone(),
                cpu_usage: report.metrics.cpu_usage,
                ram_used_mb: report.metrics.ram_used_mb,
                ram_available_mb: report.metrics.ram_available_mb,
            });
            Json(ApiResponse::ok("Report received".to_string()))
        }
        Err(e) => Json(ApiResponse::err(format!("Failed to store metrics: {}", e))),
    }
}

/// SSE stream endpoint
pub async fn sse_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    sse::sse_handler(state).await
}
