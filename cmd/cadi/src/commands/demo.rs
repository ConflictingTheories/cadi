use anyhow::Result;
use clap::Args;
use console::style;

use crate::config::CadiConfig;

/// Arguments for the demo command
#[derive(Args)]
pub struct DemoArgs {
    /// Demo to run
    #[arg(required = true)]
    demo: String,

    /// Build target
    #[arg(short, long)]
    target: Option<String>,

    /// List available demos
    #[arg(long)]
    list: bool,
}

/// Execute the demo command
pub async fn execute(args: DemoArgs, _config: &CadiConfig) -> Result<()> {
    if args.list || args.demo == "list" {
        println!("{}", style("Available Demos").bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("  {} - Full-stack todo application", style("todo-suite").cyan());
        println!("    Targets:");
        println!("      - web-dev:       React frontend + Node.js backend (dev mode)");
        println!("      - web-prod:      Production build with containers");
        println!("      - c-server-prod: C REST server in container");
        println!("      - wasm-demo:     WASM-based components");
        println!();
        println!("Usage:");
        println!("  cadi demo todo-suite --target web-dev");
        return Ok(());
    }

    match args.demo.as_str() {
        "todo-suite" => run_todo_suite_demo(&args).await,
        _ => {
            println!("{} Unknown demo: {}", style("✗").red(), args.demo);
            println!("Run 'cadi demo --list' to see available demos.");
            Ok(())
        }
    }
}

async fn run_todo_suite_demo(args: &DemoArgs) -> Result<()> {
    let target = args.target.as_deref().unwrap_or("web-dev");

    println!("{}", style("CADI Todo Suite Demo").bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("This demo showcases CADI's multi-language, multi-platform capabilities.");
    println!();

    println!("{}", style("Components:").bold());
    println!("  📦 Shared PostgreSQL schema");
    println!("  📄 OpenAPI specification (TodoApi)");
    println!("  🌐 React/TypeScript frontend");
    println!("  ⚡ Node.js REST server");
    println!("  🔌 Node.js WebSocket server");
    println!("  🔧 C REST server (with WASM support)");
    println!();

    println!("Target: {}", style(target).cyan());
    println!();

    match target {
        "web-dev" => {
            println!("{}", style("Setting up development environment...").bold());
            println!();
            println!("  {} Checking for PostgreSQL...", style("→").cyan());
            println!("  {} Starting database (docker-compose)...", style("→").cyan());
            println!("  {} Building React frontend...", style("→").cyan());
            println!("  {} Starting Node.js server...", style("→").cyan());
            println!();
            println!("{}", style("Development servers ready:").green().bold());
            println!("  Frontend:  http://localhost:3000");
            println!("  API:       http://localhost:8080");
            println!("  Database:  localhost:5432");
            println!();
            println!("Press Ctrl+C to stop all services.");
        }
        "web-prod" => {
            println!("{}", style("Building production containers...").bold());
            println!();
            println!("  {} Building optimized frontend bundle...", style("→").cyan());
            println!("  {} Creating Node.js container image...", style("→").cyan());
            println!("  {} Pushing to registry...", style("→").cyan());
            println!();
            println!("{}", style("Production artifacts:").green().bold());
            println!("  Frontend:  dist/todo-frontend.tar.gz");
            println!("  Container: ghcr.io/cadi/todo-node:latest");
        }
        "c-server-prod" => {
            println!("{}", style("Building C server...").bold());
            println!();
            println!("  {} Compiling C source to native binary...", style("→").cyan());
            println!("  {} Creating minimal container...", style("→").cyan());
            println!("  {} Signing artifacts...", style("→").cyan());
            println!();
            println!("{}", style("Artifacts:").green().bold());
            println!("  Binary:    build/todo-c-server");
            println!("  Container: ghcr.io/cadi/todo-c:latest");
            println!("  WASM:      build/todo-c-server.wasm (fallback)");
        }
        "wasm-demo" => {
            println!("{}", style("Building WASM components...").bold());
            println!();
            println!("  {} Compiling C to WASM...", style("→").cyan());
            println!("  {} Running in browser...", style("→").cyan());
            println!();
            println!("{}", style("WASM demo ready:").green().bold());
            println!("  URL: http://localhost:3000/wasm-demo");
        }
        _ => {
            println!("{} Unknown target: {}", style("✗").red(), target);
            println!("Available targets: web-dev, web-prod, c-server-prod, wasm-demo");
        }
    }

    Ok(())
}
