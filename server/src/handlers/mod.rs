//! HTTP handlers / API routes

pub mod api;
pub mod dashboard;

use axum::{
    Router,
    routing::{get, post, delete},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::AppState;

/// Build the complete application router
pub fn build_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Monitor CRUD
        .route("/monitors", get(api::list_monitors))
        .route("/monitors", post(api::create_monitor))
        .route("/monitors/:id", delete(api::delete_monitor))
        .route("/monitors/:id/results", get(api::get_monitor_results))
        // Agent endpoints
        .route("/agents", get(api::list_agents))
        .route("/agents/report", post(api::agent_report))
        // SSE endpoint
        .route("/events", get(api::sse_stream));

    Router::new()
        .nest("/api", api_routes)
        // Serve static dashboard files
        .nest_service("/", ServeDir::new("dashboard/static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
