//! CADI MCP Server
//!
//! Model Context Protocol server for LLM integration with CADI.

mod protocol;
mod tools;
mod resources;

use protocol::McpServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing (to stderr so stdout is free for JSON-RPC)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cadi_mcp_server=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting CADI MCP server");

    // Create and run the server
    let server = McpServer::new();
    
    if let Err(e) = server.run().await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
