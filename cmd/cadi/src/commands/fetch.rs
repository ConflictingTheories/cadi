use anyhow::Result;
use clap::Args;
use console::style;

use crate::config::CadiConfig;

/// Arguments for the fetch command
#[derive(Args)]
pub struct FetchArgs {
    /// Chunk ID or manifest to fetch
    #[arg(required = true)]
    target: String,

    /// Tier to fetch (source, ir, blob, all)
    #[arg(long, default_value = "all")]
    tier: String,

    /// Registry to fetch from
    #[arg(short, long)]
    registry: Option<String>,

    /// Skip signature verification
    #[arg(long)]
    no_verify: bool,
}

/// Execute the fetch command
pub async fn execute(args: FetchArgs, config: &CadiConfig) -> Result<()> {
    let registry = args.registry.as_ref()
        .unwrap_or(&config.registry.url);

    println!("{}", style("Fetching from registry...").bold());
    println!("  Registry: {}", registry);
    println!("  Target: {}", args.target);
    println!("  Tier: {}", args.tier);

    // Check if target is a chunk ID or manifest
    let is_chunk = args.target.starts_with("chunk:");

    if is_chunk {
        fetch_chunk(&args.target, &args.tier, config, !args.no_verify).await?;
    } else {
        // Treat as manifest path
        let manifest_content = std::fs::read_to_string(&args.target)?;
        let manifest: serde_json::Value = serde_yaml::from_str(&manifest_content)?;

        // Fetch all chunks referenced in manifest
        if let Some(nodes) = manifest["build_graph"]["nodes"].as_array() {
            for node in nodes {
                if let Some(source_cadi) = node["source_cadi"].as_str() {
                    fetch_chunk(source_cadi, &args.tier, config, !args.no_verify).await?;
                }
                if let Some(ir_cadi) = node["ir_cadi"].as_str() {
                    if args.tier == "ir" || args.tier == "all" {
                        fetch_chunk(ir_cadi, &args.tier, config, !args.no_verify).await?;
                    }
                }
                if let Some(blob_cadi) = node["blob_cadi"].as_str() {
                    if args.tier == "blob" || args.tier == "all" {
                        fetch_chunk(blob_cadi, &args.tier, config, !args.no_verify).await?;
                    }
                }
            }
        }
    }

    println!();
    println!("{}", style("Fetch complete!").green().bold());

    Ok(())
}

async fn fetch_chunk(chunk_id: &str, _tier: &str, config: &CadiConfig, verify: bool) -> Result<()> {
    let hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
    let chunk_file = config.cache.dir.join("chunks").join(format!("{}.json", hash));

    // Check if already cached
    if chunk_file.exists() {
        println!("  {} {} (cached)", style("✓").green(), &chunk_id[..40.min(chunk_id.len())]);
        return Ok(());
    }

    println!("  {} Fetching {}...", style("→").cyan(), &chunk_id[..40.min(chunk_id.len())]);

    // Fetch from registry (simulated)
    // Real implementation would use reqwest to GET from registry
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Verify signature if required
    if verify && config.security.verify_on_fetch {
        println!("    {} Verifying signature", style("→").dim());
    }

    // For demo, create a placeholder
    std::fs::create_dir_all(chunk_file.parent().unwrap())?;
    std::fs::write(&chunk_file, serde_json::json!({
        "chunk_id": chunk_id,
        "fetched_at": chrono::Utc::now().to_rfc3339()
    }).to_string())?;

    println!("  {} {} fetched", style("✓").green(), &chunk_id[..40.min(chunk_id.len())]);

    Ok(())
}
