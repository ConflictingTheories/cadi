use anyhow::{Result, anyhow};
use clap::Args;
use console::style;
use std::process::Stdio;
use tokio::process::Command;
use cadi_builder::engine::{BuildEngine, BuildConfig};
use cadi_core::Manifest;

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
pub async fn execute(args: DemoArgs, config: &CadiConfig) -> Result<()> {
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
        "todo-suite" => run_todo_suite_demo(&args, config).await,
        _ => {
            println!("{} Unknown demo: {}", style("✗").red(), args.demo);
            println!("Run 'cadi demo --list' to see available demos.");
            Ok(())
        }
    }
}

async fn check_tool(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn run_todo_suite_demo(args: &DemoArgs, config: &CadiConfig) -> Result<()> {
    let target = args.target.as_deref().unwrap_or("web-dev");

    println!("{}", style("CADI Todo Suite Demo").bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("This demo showcases CADI's multi-language, multi-platform capabilities.");
    println!();

    // Check prerequisites
    println!("{}", style("Checking Prerequisites:").bold());
    
    let docker_ok = check_tool("docker").await;
    println!("  {} Docker:  {}", 
        if docker_ok { style("✓").green() } else { style("✗").red() },
        if docker_ok { "Found" } else { "Not found (required for containers)" });

    let node_ok = check_tool("node").await;
    println!("  {} Node.js: {}", 
        if node_ok { style("✓").green() } else { style("✗").red() },
        if node_ok { "Found" } else { "Not found (required for web-dev)" });

    let gcc_ok = check_tool("gcc").await;
    println!("  {} GCC:     {}", 
        if gcc_ok { style("✓").green() } else { style("✗").red() },
        if gcc_ok { "Found" } else { "Not found (required for C server)" });

    println!();

    if target == "web-dev" && !node_ok {
        return Err(anyhow!("Node.js is required for the 'web-dev' target. Please install it first."));
    }

    if (target == "web-prod" || target == "c-server-prod") && !docker_ok {
        return Err(anyhow!("Docker is required for production targets. Please start Docker first."));
    }

    println!("Target: {}", style(target).cyan());
    println!();

    // Load manifest
    let manifest_path = "examples/todo-suite/todo-suite.cadi.yaml";
    if !std::path::Path::new(manifest_path).exists() {
        return Err(anyhow!("Demo manifest not found at {}", manifest_path));
    }

    let manifest_content = std::fs::read_to_string(manifest_path)?;
    let manifest: Manifest = serde_yaml::from_str(&manifest_content)?;

    println!("{}", style("Building components...").bold());

    let build_config = BuildConfig {
        parallel_jobs: config.build.parallelism,
        cache_dir: config.cache.dir.clone(),
        use_remote_cache: true,
        fail_fast: true,
        verbose: true,
    };
    
    let engine = BuildEngine::new(build_config);
    let result = engine.build(&manifest, target).await?;

    if !result.failed.is_empty() {
        return Err(anyhow!("Demo build failed."));
    }

    println!();
    println!("{}", style("Demo ready!").green().bold());
    
    match target {
        "web-dev" => {
            println!("  Frontend:  http://localhost:3000");
            println!("  API:       http://localhost:8080");
            println!();
            println!("Note: In this demo, services are built but not automatically started by the CLI.");
            println!("Run the generated artifacts to start the services.");
        }
        _ => {
            println!("Artifacts generated successfully in the CADI cache.");
        }
    }

    Ok(())
}
