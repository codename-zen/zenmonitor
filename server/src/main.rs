//! ZenMonitor Server - Main entry point
//!
//! A monitoring server that checks HTTP/HTTPS endpoints, performs ICMP pings,
//! TCP port checks, and SSL certificate expiry monitoring. Provides a web
//! dashboard with real-time updates via Server-Sent Events.

mod config;
mod db;
mod handlers;
mod models;
mod monitors;
mod sse;

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared application state passed to all handlers
pub struct AppState {
    pub db: db::Database,
    pub tx: broadcast::Sender<sse::SseEvent>,
    pub config: config::ServerConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "zenmonitor_server=info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting ZenMonitor Server...");

    // Load configuration
    let config = config::ServerConfig::load().unwrap_or_default();
    tracing::info!("Loaded configuration: listen on {}", config.listen_addr);

    // Initialize database
    let db = db::Database::new(&config.database_path)?;
    db.run_migrations()?;
    tracing::info!("Database initialized at {}", config.database_path);

    // Create broadcast channel for SSE
    let (tx, _rx) = broadcast::channel::<sse::SseEvent>(1024);

    // Build shared state
    let state = Arc::new(AppState { db, tx: tx.clone(), config: config.clone() });

    // Start background monitoring tasks
    monitors::start_all_monitors(state.clone()).await;

    // Build the Axum router
    let app = handlers::build_router(state.clone());

    // Start the server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("✅ ZenMonitor Server listening on http://{}", config.listen_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
