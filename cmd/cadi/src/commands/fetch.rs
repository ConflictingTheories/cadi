use anyhow::Result;
use clap::Args;
use console::style;
use cadi_registry::client::{RegistryClient, RegistryConfig};

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

async fn fetch_chunk(chunk_id: &str, _tier: &str, config: &CadiConfig, _verify: bool) -> Result<()> {
    // Initialize registry client
    let registry_url = config.registry.url.clone();
    let reg_config = RegistryConfig {
        url: registry_url.clone(),
        token: config.auth.token.clone(),
        timeout: std::time::Duration::from_secs(30),
        verify_tls: true,
        max_concurrent: 4,
    };
    
    let client = RegistryClient::new(reg_config)
        .map_err(|e| anyhow::anyhow!("Failed to create registry client: {}", e))?;

    // Check if chunk already exists locally
    let cache_dir = config.cache.dir.join("chunks");
    let hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
    let chunk_file = cache_dir.join(format!("{}.bin", hash));

    if chunk_file.exists() {
        println!("  {} {} (cached)", style("✓").green(), &chunk_id[..40.min(chunk_id.len())]);
        return Ok(());
    }

    println!("  {} Fetching {}...", style("→").cyan(), &chunk_id[..40.min(chunk_id.len())]);

    // Fetch from registry
    match client.fetch_chunk(chunk_id).await {
        Ok(data) => {
            // Save to local cache
            std::fs::create_dir_all(&cache_dir)?;
            std::fs::write(&chunk_file, &data)?;
            
            // Also save metadata
            let meta_file = cache_dir.join(format!("{}.json", hash));
            let metadata = serde_json::json!({
                "chunk_id": chunk_id,
                "size": data.len(),
                "fetched_at": chrono::Utc::now().to_rfc3339(),
                "registry": registry_url
            });
            std::fs::write(&meta_file, serde_json::to_string_pretty(&metadata)?)?;
            
            println!("  {} {} fetched ({} bytes)", style("✓").green(), &chunk_id[..40.min(chunk_id.len())], data.len());
        }
        Err(e) => {
            eprintln!("  {} Failed to fetch: {}", style("✗").red(), e);
            return Err(anyhow::anyhow!("Fetch failed: {}", e));
        }
    }

    Ok(())
}
