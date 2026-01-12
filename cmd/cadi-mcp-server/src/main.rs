//! CADI MCP Server
//!
//! Model Context Protocol server for LLM integration with CADI.
//!
//! ## Transport Modes
//! - **stdio** (default): Reads JSON-RPC from stdin, writes to stdout
//! - **http**: Runs as HTTP server for Docker/container deployment
//!
//! ## Token-Saving Features
//! - Pre-built prompts that guide agents to use CADI-first workflow
//! - Resources that document efficient usage patterns
//! - Tools optimized for minimal token consumption

mod protocol;
mod tools;
mod resources;
mod prompts;

use clap::Parser;
use protocol::McpServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// CADI MCP Server - Model Context Protocol server for LLM integration
#[derive(Parser, Debug)]
#[command(name = "cadi-mcp-server")]
#[command(version, about, long_about = None)]
struct Args {
    /// Transport mode: "stdio" or "http"
    #[arg(short, long, default_value = "stdio", env = "CADI_MCP_TRANSPORT")]
    transport: String,

    /// Bind address for HTTP mode (ignored in stdio mode)
    #[arg(short, long, default_value = "0.0.0.0:9090", env = "CADI_MCP_BIND_ADDRESS")]
    bind: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing (to stderr so stdout is free for JSON-RPC in stdio mode)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cadi_mcp_server=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    let args = Args::parse();

    tracing::info!("Starting CADI MCP server in {} mode", args.transport);

    // Create the server
    let server = McpServer::new();

    let result = match args.transport.as_str() {
        "stdio" => server.run_stdio().await,
        "http" => server.run_http(&args.bind).await,
        other => {
            tracing::error!("Unknown transport: {}. Use 'stdio' or 'http'", other);
            std::process::exit(1);
        }
    };
    
    if let Err(e) = result {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
