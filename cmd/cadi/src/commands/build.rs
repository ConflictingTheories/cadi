use anyhow::Result;
use clap::Args;
use console::style;
use std::path::PathBuf;
use cadi_builder::engine::{BuildEngine, BuildConfig};
use cadi_core::Manifest;

use crate::config::CadiConfig;

/// Arguments for the build command
#[derive(Args)]
pub struct BuildArgs {
    /// Path to manifest file
    #[arg(required = true)]
    manifest: PathBuf,

    /// Build target name
    #[arg(short, long)]
    target: Option<String>,

    /// Preferred representation
    #[arg(long)]
    prefer: Option<String>,

    /// Force rebuild (ignore cache)
    #[arg(long)]
    force: bool,

    /// Only verify build plan, don't execute
    #[arg(long)]
    dry_run: bool,
}

/// Execute the build command
pub async fn execute(args: BuildArgs, config: &CadiConfig) -> Result<()> {
    println!("{}", style("Building from manifest...").bold());
    println!("  Manifest: {}", args.manifest.display());
    
    if let Some(target) = &args.target {
        println!("  Target: {}", target);
    }

    // Load manifest from file
    let manifest_content = std::fs::read_to_string(&args.manifest)?;
    let manifest: Manifest = if args.manifest.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
        serde_yaml::from_str(&manifest_content)?
    } else {
        serde_json::from_str(&manifest_content)?
    };

    println!("  Application: {}", manifest.application.name);
    println!("  Version: {}", manifest.application.version.as_deref().unwrap_or("0.1.0"));

    let target_name = args.target.as_deref().unwrap_or("dev");
    
    println!();
    println!("{}", style("Build Plan:").bold());

    // Show nodes to be built
    for (idx, node) in manifest.build_graph.nodes.iter().enumerate().take(10) {
        println!("  {} Build node: {} ({})", style("→").cyan(), node.id, 
            if node.source_cadi.is_some() { "source" } else { "derived" });
    }
    
    if manifest.build_graph.nodes.len() > 10 {
        println!("  ... and {} more nodes", manifest.build_graph.nodes.len() - 10);
    }

    if args.dry_run {
        println!();
        println!("{}", style("Dry run - no changes made.").yellow());
        return Ok(());
    }

    println!();
    println!("{}", style("Executing build...").bold());

    // Create and run build engine
    let build_config = BuildConfig {
        parallel_jobs: config.build.parallelism,
        cache_dir: config.cache.dir.clone(),
        use_remote_cache: true,
        fail_fast: false,
        verbose: true,
    };
    
    let engine = BuildEngine::new(build_config);
    let start = std::time::Instant::now();
    
    match engine.build(&manifest, target_name).await {
        Ok(result) => {
            println!("  {} Successfully built {} chunks", style("✓").green(), result.built.len());
            if !result.cached.is_empty() {
                println!("  {} Using cache for {} chunks", style("✓").green(), result.cached.len());
            }
            
            if !result.failed.is_empty() {
                println!("  {} {} builds failed", style("✗").red(), result.failed.len());
                for failure in &result.failed {
                    println!("    - {}: {}", failure.chunk_id, failure.error);
                }
                return Err(anyhow::anyhow!("Some builds failed"));
            }
            
            let elapsed = start.elapsed().as_secs_f64();
            println!();
            println!("{}", style(format!("Build complete in {:.2}s!", elapsed)).green().bold());
        }
        Err(e) => {
            eprintln!("  {} Build failed: {}", style("✗").red(), e);
            return Err(anyhow::anyhow!("Build failed: {}", e));
        }
    }

    Ok(())
}

