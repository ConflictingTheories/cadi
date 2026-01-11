use anyhow::Result;
use clap::Args;
use console::style;
use std::path::PathBuf;

use crate::config::CadiConfig;

/// Arguments for the plan command
#[derive(Args)]
pub struct PlanArgs {
    /// Path to manifest file
    #[arg(required = true)]
    manifest: PathBuf,

    /// Build target name
    #[arg(short, long)]
    target: Option<String>,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    format: String,
}

/// Execute the plan command
pub async fn execute(args: PlanArgs, config: &CadiConfig) -> Result<()> {
    // Load manifest
    let manifest_content = std::fs::read_to_string(&args.manifest)?;
    let manifest: serde_json::Value = if args.manifest.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
        serde_yaml::from_str(&manifest_content)?
    } else {
        serde_json::from_str(&manifest_content)?
    };

    let app_name = manifest["application"]["name"].as_str().unwrap_or("unknown");
    let target_name = args.target.as_deref().unwrap_or("dev");

    if args.format == "json" {
        // JSON output
        let plan = build_plan_json(&manifest, target_name, config)?;
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    // Text output
    println!("{}", style("Build Plan").bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Application: {}", style(app_name).cyan());
    println!("Target:      {}", style(target_name).cyan());
    println!();

    // Find target
    let targets = manifest["build_targets"].as_array();
    let build_target = targets
        .and_then(|t| t.iter().find(|bt| bt["name"].as_str() == Some(target_name)));

    if build_target.is_none() {
        println!("{}", style("Target not found!").red());
        return Ok(());
    }

    let build_target = build_target.unwrap();
    let platform = build_target["platform"].as_str().unwrap_or("any");
    println!("Platform:    {}", platform);
    println!();

    println!("{}", style("Operations:").bold());
    println!();

    let nodes = manifest["build_graph"]["nodes"].as_array();
    let target_nodes = build_target["nodes"].as_array();

    let mut total_cached = 0;
    let mut total_fetch = 0;
    let mut total_build = 0;
    let mut estimated_size: u64 = 0;

    if let (Some(nodes), Some(target_nodes)) = (nodes, target_nodes) {
        for target_node in target_nodes {
            let node_id = target_node["id"].as_str().unwrap_or("");
            
            if let Some(node) = nodes.iter().find(|n| n["id"].as_str() == Some(node_id)) {
                let source_cadi = node["source_cadi"].as_str();
                let has_cached = check_cached(source_cadi, config);
                let has_registry = false; // Would check registry

                let (action, icon) = if has_cached {
                    total_cached += 1;
                    ("cached", style("◉").green())
                } else if has_registry {
                    total_fetch += 1;
                    estimated_size += 1024 * 100; // Estimate 100KB
                    ("fetch", style("↓").blue())
                } else {
                    total_build += 1;
                    ("build", style("⚙").yellow())
                };

                println!("  {} {} {}", icon, style(node_id).bold(), style(format!("[{}]", action)).dim());
                
                if let Some(chunk_id) = source_cadi {
                    println!("      chunk: {}", &chunk_id[..50.min(chunk_id.len())]);
                }
            }
        }
    }

    println!();
    println!("{}", style("Summary:").bold());
    println!("  Cached:  {} items", total_cached);
    println!("  Fetch:   {} items (~{} KB)", total_fetch, estimated_size / 1024);
    println!("  Build:   {} items", total_build);
    println!();

    if total_build > 0 {
        println!("Estimated build time: ~{} seconds", total_build * 5);
    }
    if total_fetch > 0 {
        println!("Estimated download:   ~{} KB", estimated_size / 1024);
    }

    Ok(())
}

fn check_cached(chunk_id: Option<&str>, config: &CadiConfig) -> bool {
    if let Some(id) = chunk_id {
        let hash = id.strip_prefix("chunk:sha256:").unwrap_or(id);
        let chunk_file = config.cache.dir.join("chunks").join(format!("{}.json", hash));
        chunk_file.exists()
    } else {
        false
    }
}

fn build_plan_json(manifest: &serde_json::Value, target_name: &str, config: &CadiConfig) -> Result<serde_json::Value> {
    let mut operations = Vec::new();
    
    let targets = manifest["build_targets"].as_array();
    let build_target = targets
        .and_then(|t| t.iter().find(|bt| bt["name"].as_str() == Some(target_name)));

    if let Some(build_target) = build_target {
        let nodes = manifest["build_graph"]["nodes"].as_array();
        let target_nodes = build_target["nodes"].as_array();

        if let (Some(nodes), Some(target_nodes)) = (nodes, target_nodes) {
            for target_node in target_nodes {
                let node_id = target_node["id"].as_str().unwrap_or("");
                
                if let Some(node) = nodes.iter().find(|n| n["id"].as_str() == Some(node_id)) {
                    let source_cadi = node["source_cadi"].as_str();
                    let has_cached = check_cached(source_cadi, config);

                    operations.push(serde_json::json!({
                        "node_id": node_id,
                        "chunk_id": source_cadi,
                        "action": if has_cached { "cached" } else { "build" },
                        "cached": has_cached
                    }));
                }
            }
        }
    }

    Ok(serde_json::json!({
        "application": manifest["application"]["name"],
        "target": target_name,
        "operations": operations
    }))
}
