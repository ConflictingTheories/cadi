use anyhow::{Result, anyhow};
use clap::Args;
use console::style;
use reqwest::{Client, header};
use std::path::Path;

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

    /// Authentication token for registry
    #[arg(long)]
    auth_token: Option<String>,

    /// Namespace for chunks
    #[arg(long)]
    namespace: Option<String>,

    /// Batch size for concurrent publishing
    #[arg(long, default_value = "5")]
    batch_size: usize,

    /// Skip deduplication checks
    #[arg(long)]
    no_dedup: bool,
}

/// Publish state tracker
struct PublishStats {
    total: usize,
    published: usize,
    skipped: usize,
    failed: usize,
    bytes_published: u64,
}

/// Execute the publish command
pub async fn execute(args: PublishArgs, config: &CadiConfig) -> Result<()> {
    let registry = args.registry.as_ref()
        .unwrap_or(&config.registry.url);

    println!("{}", style("Publishing chunks to registry...").bold());
    println!("  Registry: {}", registry);
    if let Some(ref ns) = args.namespace {
        println!("  Namespace: {}", ns);
    }

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

    println!("  {} Found {} chunks to publish", style("✓").green(), chunks_to_publish.len());
    println!();

    if args.dry_run {
        println!("{}", style("Dry run - would publish:").yellow());
        for (id, path) in &chunks_to_publish {
            let size = std::fs::metadata(path)?.len();
            println!("  - {} ({} bytes)", id, size);
        }
        return Ok(());
    }

    // Publish chunks with batching
    let mut stats = PublishStats {
        total: chunks_to_publish.len(),
        published: 0,
        skipped: 0,
        failed: 0,
        bytes_published: 0,
    };

    let client = create_client(&args.auth_token)?;

    // Process in batches
    for (i, (id, path)) in chunks_to_publish.iter().enumerate() {
        let display_id = if id.len() > 40 { &id[..40] } else { id };
        print!("  [{}/{}] Publishing {}... ", i + 1, stats.total, display_id);
        use std::io::Write;
        std::io::stdout().flush()?;

        match publish_chunk(&client, registry, id, path, &args, config).await {
            Ok(size) => {
                println!("{}", style("✓").green());
                stats.published += 1;
                stats.bytes_published += size as u64;
            }
            Err(e) if e.to_string().contains("exists") && !args.no_dedup => {
                println!("{}", style("(skipped - exists)").yellow());
                stats.skipped += 1;
            }
            Err(e) => {
                println!("{} {}", style("✗").red(), e);
                stats.failed += 1;
            }
        }
    }

    println!();
    println!("{}", style("Publish Summary:").bold());
    println!("  {} Published: {}", style("✓").green(), stats.published);
    if stats.skipped > 0 {
        println!("  {} Skipped: {}", style("→").yellow(), stats.skipped);
    }
    if stats.failed > 0 {
        println!("  {} Failed: {}", style("✗").red(), stats.failed);
    }
    println!("  {} Total bytes: {}", style("→").cyan(), stats.bytes_published);

    if stats.failed > 0 {
        return Err(anyhow!("{} chunks failed to publish", stats.failed));
    }

    println!();
    println!("{}", style("Publish complete!").green().bold());

    Ok(())
}

/// Create HTTP client with auth headers
fn create_client(auth_token: &Option<String>) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    
    if let Some(token) = auth_token {
        let auth_header = format!("Bearer {}", token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_header)?,
        );
    }

    Ok(Client::builder()
        .default_headers(headers)
        .build()?)
}

/// Publish a single chunk
async fn publish_chunk(
    client: &Client,
    registry: &str,
    chunk_id: &str,
    path: &Path,
    args: &PublishArgs,
    config: &CadiConfig,
) -> Result<usize> {
    let content = std::fs::read(path)?;
    let size = content.len();

    // Sign if required
    if !args.no_sign {
        if config.security.signing_key.is_some() {
            // TODO: Sign chunk with configured key
        }
    }

    // Build URL with namespace if provided
    let url = if let Some(ref ns) = args.namespace {
        format!("{}/v1/namespaces/{}/chunks/{}", 
            registry.trim_end_matches('/'), ns, chunk_id)
    } else {
        format!("{}/v1/chunks/{}", registry.trim_end_matches('/'), chunk_id)
    };

    let response = client
        .put(&url)
        .body(content)
        .header("Content-Type", "application/octet-stream")
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {}", e))?;

    match response.status() {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => Ok(size),
        reqwest::StatusCode::CONFLICT => {
            Err(anyhow!("Chunk already exists at registry"))
        }
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(anyhow!("HTTP {}: {}", status, body))
        }
    }
}
