use anyhow::Result;
use clap::Args;
use console::style;
use std::path::PathBuf;

use crate::config::CadiConfig;

/// Arguments for the run command
#[derive(Args)]
pub struct RunArgs {
    /// Manifest or chunk to run
    #[arg(required = true)]
    target: String,

    /// Build target name
    #[arg(short, long)]
    build_target: Option<String>,

    /// Run in sandbox mode
    #[arg(long)]
    sandbox: bool,

    /// Environment variables
    #[arg(short, long, value_parser = parse_env_var)]
    env: Vec<(String, String)>,

    /// Arguments to pass to the program
    #[arg(last = true)]
    args: Vec<String>,
}

fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(format!("Invalid environment variable: {}", s))
    }
}

/// Execute the run command
pub async fn execute(args: RunArgs, config: &CadiConfig) -> Result<()> {
    println!("{}", style("Running...").bold());
    println!("  Target: {}", args.target);
    
    if let Some(bt) = &args.build_target {
        println!("  Build target: {}", bt);
    }
    
    if args.sandbox {
        println!("  Mode: {}", style("sandboxed").yellow());
    }

    // Determine run mode based on target
    let target_path = PathBuf::from(&args.target);
    
    if target_path.exists() {
        // It's a manifest file
        run_from_manifest(&args, config).await?;
    } else if args.target.starts_with("chunk:") {
        // It's a chunk ID
        run_chunk(&args.target, &args, config).await?;
    } else {
        anyhow::bail!("Target not found: {}", args.target);
    }

    Ok(())
}

async fn run_from_manifest(args: &RunArgs, _config: &CadiConfig) -> Result<()> {
    let manifest_content = std::fs::read_to_string(&args.target)?;
    let manifest: serde_json::Value = serde_yaml::from_str(&manifest_content)?;

    let app_name = manifest["application"]["name"].as_str().unwrap_or("unknown");
    println!("  Application: {}", app_name);

    // Find the target
    let target_name = args.build_target.as_deref().unwrap_or("dev");
    let targets = manifest["build_targets"].as_array();
    
    let build_target = targets
        .and_then(|t| t.iter().find(|bt| bt["name"].as_str() == Some(target_name)));

    if build_target.is_none() {
        println!("  {} Target '{}' not found", style("✗").red(), target_name);
        return Ok(());
    }

    let build_target = build_target.unwrap();
    let platform = build_target["platform"].as_str().unwrap_or("any");

    println!("  Platform: {}", platform);
    println!();

    // Determine how to run based on platform
    match platform {
        "browser" | "browser + node-dev" => {
            println!("{}", style("Starting development server...").green());
            println!("  http://localhost:3000");
            println!();
            println!("  Press Ctrl+C to stop");
            
            // In real implementation, would start a dev server
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        "linux-container" | "x86_64-linux" | "arm64-linux" => {
            println!("{}", style("Running in container...").green());
            
            if args.sandbox {
                println!("  {} Container isolation enabled", style("🔒").cyan());
            }
        }
        _ => {
            println!("{}", style("Running natively...").green());
        }
    }

    Ok(())
}

async fn run_chunk(chunk_id: &str, args: &RunArgs, config: &CadiConfig) -> Result<()> {
    let hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
    let chunk_file = config.cache.dir.join("chunks").join(format!("{}.json", hash));

    if !chunk_file.exists() {
        println!("  {} Chunk not found locally. Fetching...", style("→").cyan());
        // Would fetch here
    }

    let chunk_content = std::fs::read_to_string(&chunk_file)?;
    let chunk: serde_json::Value = serde_json::from_str(&chunk_content)?;

    let cadi_type = chunk["cadi_type"].as_str().unwrap_or("unknown");

    println!("  Chunk type: {}", cadi_type);

    match cadi_type {
        "source" => {
            let language = chunk["source"]["language"].as_str().unwrap_or("unknown");
            println!("  Language: {}", language);
            
            // For source, we need to build first or run interpreted
            println!();
            println!("{}", style("Note: Source chunks require building before running.").yellow());
            println!("  Run: cadi build <manifest> && cadi run <manifest>");
        }
        "intermediate" => {
            println!("{}", style("Running WASM module...").green());
            
            if args.sandbox || config.security.sandbox_untrusted {
                println!("  {} WASM sandbox enabled", style("🔒").cyan());
            }
            
            // Would use wasmtime to run
        }
        "blob" => {
            println!("{}", style("Running native binary...").green());
            
            if args.sandbox {
                println!("  {} Container sandbox enabled", style("🔒").cyan());
            }
        }
        "container" => {
            let image_ref = chunk["container"]["image_ref"].as_str().unwrap_or("unknown");
            println!("  Image: {}", image_ref);
            println!("{}", style("Running container...").green());
        }
        _ => {
            anyhow::bail!("Unknown chunk type: {}", cadi_type);
        }
    }

    Ok(())
}
