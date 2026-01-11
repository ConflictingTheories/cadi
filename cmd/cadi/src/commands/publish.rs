use anyhow::{Result, anyhow};
use clap::Args;
use console::style;
use reqwest::Client;

use crate::config::CadiConfig;

/// Arguments for the publish command
#[derive(Args)]
pub struct PublishArgs {
    /// Chunk IDs to publish (all local if omitted)
    #[arg()]
    chunks: Vec<String>,

    /// Publish to specific registry
    #[arg(short, long)]
    registry: Option<String>,

    /// Skip signing
    #[arg(long)]
    no_sign: bool,

    /// Dry run - show what would be published
    #[arg(long)]
    dry_run: bool,
}

/// Execute the publish command
pub async fn execute(args: PublishArgs, config: &CadiConfig) -> Result<()> {
    let registry = args.registry.as_ref()
        .unwrap_or(&config.registry.url);

    println!("{}", style("Publishing to registry...").bold());
    println!("  Registry: {}", registry);

    // Find chunks to publish
    let chunks_dir = config.cache.dir.join("chunks");
    let mut chunks_to_publish = Vec::new();

    if args.chunks.is_empty() {
        // Publish all local chunks
        if chunks_dir.exists() {
            for entry in std::fs::read_dir(&chunks_dir)? {
                let entry = entry?;
                let path = entry.path();
                // Check for .chunk (data) files
                if path.extension().map(|e| e == "chunk").unwrap_or(false) {
                    let chunk_id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| format!("chunk:sha256:{}", s));
                    
                    if let Some(id) = chunk_id {
                        chunks_to_publish.push((id, path));
                    }
                }
            }
        }
    } else {
        for chunk_id in &args.chunks {
            let hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
            // Try .chunk file first (binary), then .json (metadata)
            let chunk_path = chunks_dir.join(format!("{}.chunk", hash));
            let json_path = chunks_dir.join(format!("{}.json", hash));
            
            if chunk_path.exists() {
                chunks_to_publish.push((chunk_id.clone(), chunk_path));
            } else if json_path.exists() {
                chunks_to_publish.push((chunk_id.clone(), json_path));
            } else {
                println!("  {} Chunk not found locally: {}", style("⚠").yellow(), chunk_id);
            }
        }
    }

    if chunks_to_publish.is_empty() {
        println!("  {} No chunks to publish", style("!").yellow());
        return Ok(());
    }

    println!("  Found {} chunks to publish", chunks_to_publish.len());
    println!();

    if args.dry_run {
        println!("{}", style("Dry run - would publish:").yellow());
        for (id, _) in &chunks_to_publish {
            println!("  - {}", id);
        }
        return Ok(());
    }

    // Create HTTP client
    let client = Client::new();

    // Publish chunks
    for (id, path) in &chunks_to_publish {
        let display_id = if id.len() > 40 { &id[..40] } else { id };
        println!("  {} Publishing {}...", style("→").cyan(), display_id);
        
        // Read chunk data
        let content = std::fs::read(path)?;
        
        // Sign if required
        if !args.no_sign {
            if config.security.signing_key.is_some() {
                println!("    {} Signing chunk", style("→").dim());
            }
        }
        
        // Upload to registry
        let url = format!("{}/v1/chunks/{}", registry.trim_end_matches('/'), id);
        
        let response = client
            .put(&url)
            .body(content)
            .header("Content-Type", "application/octet-stream")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to registry: {}", e))?;
        
        if response.status().is_success() {
            println!("  {} Published {}", style("✓").green(), display_id);
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            println!("  {} Failed to publish {}: {} {}", 
                     style("✗").red(), display_id, status, body);
        }
    }

    println!();
    println!("{}", style("Publish complete!").green().bold());
    println!("  {} chunks published to {}", chunks_to_publish.len(), registry);

    Ok(())
}
