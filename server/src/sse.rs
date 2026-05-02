//! Server-Sent Events (SSE) module for real-time updates

use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::AppState;

/// Events that can be broadcast via SSE
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SseEvent {
    /// A monitor check completed
    CheckResult {
        monitor_id: String,
        status: String,
        response_time_ms: Option<f64>,
        message: Option<String>,
    },
    /// An agent reported metrics
    AgentMetrics {
        agent_id: String,
        hostname: String,
        cpu_usage: f64,
        ram_used_mb: f64,
        ram_available_mb: f64,
    },
    /// A monitor was added or removed
    MonitorUpdate {
        action: String,
        monitor_id: String,
    },
    /// Agent came online or went offline
    AgentStatus {
        agent_id: String,
        status: String,
    },
}

/// SSE handler - clients connect here for real-time updates
pub async fn sse_handler(
    state: Arc<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                let json = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().data(json)))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}
