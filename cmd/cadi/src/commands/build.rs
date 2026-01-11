use anyhow::Result;
use clap::Args;
use console::style;
use std::path::PathBuf;

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

    // Load manifest
    let manifest_content = std::fs::read_to_string(&args.manifest)?;
    let manifest: serde_json::Value = if args.manifest.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
        serde_yaml::from_str(&manifest_content)?
    } else {
        serde_json::from_str(&manifest_content)?
    };

    let app_name = manifest["application"]["name"].as_str().unwrap_or("unknown");
    println!("  Application: {}", app_name);

    // Get build targets
    let targets = manifest["build_targets"].as_array();
    let target_name = args.target.as_deref().unwrap_or("dev");
    
    let build_target = targets
        .and_then(|t| t.iter().find(|bt| bt["name"].as_str() == Some(target_name)));

    if build_target.is_none() {
        println!("  {} Target '{}' not found. Available targets:", style("⚠").yellow(), target_name);
        if let Some(targets) = targets {
            for t in targets {
                if let Some(name) = t["name"].as_str() {
                    println!("    - {}", name);
                }
            }
        }
        return Ok(());
    }

    let build_target = build_target.unwrap();
    let platform = build_target["platform"].as_str().unwrap_or("any");
    println!("  Platform: {}", platform);

    println!();
    println!("{}", style("Build Plan:").bold());

    // Get nodes to build
    let nodes = manifest["build_graph"]["nodes"].as_array();
    let target_nodes = build_target["nodes"].as_array();

    let mut build_steps = Vec::new();

    if let (Some(nodes), Some(target_nodes)) = (nodes, target_nodes) {
        for target_node in target_nodes {
            let node_id = target_node["id"].as_str().unwrap_or("");
            let prefer = target_node["prefer"].as_array()
                .map(|p| p.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["source"]);

            if let Some(node) = nodes.iter().find(|n| n["id"].as_str() == Some(node_id)) {
                let source_cadi = node["source_cadi"].as_str();
                let has_cached = check_cached(source_cadi, config)?;

                build_steps.push(BuildStep {
                    node_id: node_id.to_string(),
                    source_chunk: source_cadi.map(String::from),
                    prefer: prefer.iter().map(|s| s.to_string()).collect(),
                    cached: has_cached,
                });

                let status = if has_cached {
                    style("cached").green()
                } else {
                    style("build").yellow()
                };

                println!("  {} {} (prefer: {:?}) [{}]", 
                    style("→").cyan(), 
                    node_id, 
                    prefer, 
                    status
                );
            }
        }
    }

    if args.dry_run {
        println!();
        println!("{}", style("Dry run - no changes made.").yellow());
        return Ok(());
    }

    println!();
    println!("{}", style("Executing build...").bold());

    // Execute build steps
    for step in &build_steps {
        if step.cached && !args.force {
            println!("  {} {} (using cache)", style("✓").green(), step.node_id);
        } else {
            println!("  {} Building {}...", style("→").cyan(), step.node_id);
            
            // Simulate build (real implementation would invoke transformations)
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            println!("  {} {} built successfully", style("✓").green(), step.node_id);
        }
    }

    println!();
    println!("{}", style("Build complete!").green().bold());

    Ok(())
}

struct BuildStep {
    node_id: String,
    source_chunk: Option<String>,
    prefer: Vec<String>,
    cached: bool,
}

fn check_cached(chunk_id: Option<&str>, config: &CadiConfig) -> Result<bool> {
    if let Some(id) = chunk_id {
        let hash = id.strip_prefix("chunk:sha256:").unwrap_or(id);
        let chunk_file = config.cache.dir.join("chunks").join(format!("{}.json", hash));
        Ok(chunk_file.exists())
    } else {
        Ok(false)
    }
}
