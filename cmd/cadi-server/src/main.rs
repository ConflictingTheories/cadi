//! CADI Registry Server
//!
//! HTTP server for hosting a CADI registry.

mod handlers;
mod state;
mod routes;

use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cadi_server=info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = state::ServerConfig::from_env();
    let state = state::AppState::new(config.clone()).await;

    // Build the router
    let app = Router::new()
        .merge(routes::api_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start the server
    let addr: SocketAddr = config.bind_address.parse()
        .expect("Invalid bind address");
    
    tracing::info!("Starting CADI server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await
        .expect("Failed to bind");
    
    axum::serve(listener, app).await
        .expect("Server error");
}
