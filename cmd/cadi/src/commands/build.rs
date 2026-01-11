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
            let elapsed = start.elapsed().as_secs_f64();
            
            println!();
            println!("{}", style("═══════════════════════════════════════").green());
            println!("{}", style("Build Summary").green().bold());
            println!("{}", style("═══════════════════════════════════════").green());
            
            if !result.cached.is_empty() {
                println!("  {} {} chunk(s) reused from cache", 
                    style("✓").green(), 
                    style(result.cached.len()).cyan().bold());
                for chunk_id in &result.cached {
                    let display_id = if chunk_id.len() > 60 { 
                        format!("{}...{}", &chunk_id[..30], &chunk_id[chunk_id.len()-10..])
                    } else { 
                        chunk_id.clone() 
                    };
                    println!("    • {}", display_id);
                }
            }
            
            if !result.built.is_empty() {
                println!("  {} {} chunk(s) built fresh", 
                    style("✓").yellow(), 
                    style(result.built.len()).cyan().bold());
                for chunk_id in &result.built {
                    let display_id = if chunk_id.len() > 60 { 
                        format!("{}...{}", &chunk_id[..30], &chunk_id[chunk_id.len()-10..])
                    } else { 
                        chunk_id.clone() 
                    };
                    println!("    • {}", display_id);
                }
            }
            
            if !result.failed.is_empty() {
                println!("  {} {} build(s) failed", 
                    style("✗").red(), 
                    style(result.failed.len()).red().bold());
                for failure in &result.failed {
                    println!("    • {}: {}", failure.chunk_id, failure.error);
                }
                println!();
                return Err(anyhow::anyhow!("Some builds failed"));
            }
            
            println!();
            println!("  {} Total time: {:.2}s", style("⏱").cyan(), elapsed);
            
            if !result.cached.is_empty() {
                let cache_ratio = result.cached.len() as f64 / 
                    (result.built.len() + result.cached.len()) as f64;
                println!("  {} Cache hit rate: {:.0}%", 
                    style("📊").cyan(), 
                    cache_ratio * 100.0);
                
                let estimated_saved = result.cached.len() as f64 * 2.5; // Estimate 2.5s per cached chunk
                println!("  {} Time saved by cache: ~{:.1}s", 
                    style("⚡").cyan(), 
                    estimated_saved);
            }
            
            println!("{}", style("═══════════════════════════════════════").green());
            println!();
            println!("{}", style("Build complete!").green().bold());
        }
        Err(e) => {
            eprintln!("  {} Build failed: {}", style("✗").red(), e);
            return Err(anyhow::anyhow!("Build failed: {}", e));
        }
    }

    Ok(())
}

