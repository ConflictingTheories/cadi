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

    // Map from hash -> (chunk_path, metadata_path)
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Default, Clone)]
    struct PublishItem {
        id: String,
        chunk_path: Option<PathBuf>,
        meta_path: Option<PathBuf>,
    }

    let mut map: HashMap<String, PublishItem> = HashMap::new();

    if chunks_dir.exists() {
        for entry in std::fs::read_dir(&chunks_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "chunk" || ext == "json" {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let id = format!("chunk:sha256:{}", stem);
                        let item = map.entry(stem.to_string()).or_insert(PublishItem::default());
                        item.id = id.clone();
                        if ext == "chunk" {
                            item.chunk_path = Some(path.clone());
                        } else {
                            item.meta_path = Some(path.clone());
                        }
                    }
                }
            }
        }
    }

    if !args.chunks.is_empty() {
        // Ensure requested chunks exist (at least one of chunk or metadata)
        let mut filtered = HashMap::new();
        for chunk_id in &args.chunks {
            let hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
            if let Some(item) = map.get(hash) {
                filtered.insert(hash.to_string(), item.clone());
            } else {
                let chunk_path = chunks_dir.join(format!("{}.chunk", hash));
                let json_path = chunks_dir.join(format!("{}.json", hash));
                if chunk_path.exists() || json_path.exists() {
                    let mut item = PublishItem { id: format!("chunk:sha256:{}", hash), chunk_path: None, meta_path: None };
                    if chunk_path.exists() { item.chunk_path = Some(chunk_path); }
                    if json_path.exists() { item.meta_path = Some(json_path); }
                    filtered.insert(hash.to_string(), item);
                } else {
                    println!("  {} Chunk not found locally: {}", style("⚠").yellow(), chunk_id);
                }
            }
        }
        map = filtered.into_iter().map(|(k, v)| (k, v)).collect();
    }

    let chunks_to_publish: Vec<PublishItem> = map.into_values().collect();

    if chunks_to_publish.is_empty() {
        println!("  {} No chunks to publish", style("!").yellow());
        return Ok(());
    }

    println!("  {} Found {} chunks to publish", style("✓").green(), chunks_to_publish.len());
    println!();

    if args.dry_run {
        println!("{}", style("Dry run - would publish:").yellow());
        for item in &chunks_to_publish {
            let mut parts = vec![];
            if let Some(chunk_path) = &item.chunk_path {
                let size = std::fs::metadata(chunk_path)?.len();
                parts.push(format!("data ({} bytes)", size));
            }
            if item.meta_path.is_some() {
                parts.push("metadata".to_string());
            }
            println!("  - {}: {}", item.id, parts.join(", "));
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
    for (i, item) in chunks_to_publish.iter().enumerate() {
        let display_id = if item.id.len() > 40 { &item.id[..40] } else { &item.id };
        print!("  [{}/{}] Publishing {}... ", i + 1, stats.total, display_id);
        use std::io::Write;
        std::io::stdout().flush()?;

        // First publish data chunk if present
        if let Some(chunk_path) = &item.chunk_path {
            match publish_chunk(&client, registry, &item.id, chunk_path, &args, config).await {
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
        } else {
            // No chunk data to publish
            println!("{}", style("(no data)").yellow());
        }

        // Then publish metadata if present
        if let Some(meta_path) = &item.meta_path {
            match publish_metadata(&client, registry, &item.id, meta_path, &args, config).await {
                Ok(_) => {
                    println!("  {} metadata updated", style("✓").green());
                }
                Err(e) => {
                    println!("  {} metadata failed: {}", style("✗").red(), e);
                    stats.failed += 1;
                }
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
        if let Some(key_path) = &config.security.signing_key {
            if key_path.exists() {
                let key_content = std::fs::read_to_string(key_path)?;
                let signature = sign_content(&content, &key_content)?;
                
                // Update metadata with signature
                let hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
                let metadata_path = config.cache.dir.join("chunks").join(format!("{}.json", hash));
                if metadata_path.exists() {
                    let mut meta_content: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&metadata_path)?)?;
                    if let Some(signatures) = meta_content.get_mut("signatures").and_then(|s| s.as_array_mut()) {
                        signatures.push(serde_json::Value::String(signature));
                    } else {
                        meta_content["signatures"] = serde_json::json!([signature]);
                    }
                    std::fs::write(&metadata_path, serde_json::to_string_pretty(&meta_content)?)?;
                }
            }
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

/// Publish metadata for a chunk (JSON)
async fn publish_metadata(
    client: &Client,
    registry: &str,
    chunk_id: &str,
    meta_path: &Path,
    _args: &PublishArgs,
    _config: &CadiConfig,
) -> Result<()> {
    let content = std::fs::read_to_string(meta_path)?;

    // Build URL with namespace if provided
    let url = format!("{}/v1/chunks/{}/meta", registry.trim_end_matches('/'), chunk_id);

    let response = client
        .put(&url)
        .body(content)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {}", e))?;

    match response.status() {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => Ok(()),
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(anyhow!("HTTP {}: {}", status, body))
        }
    }
}

fn sign_content(content: &[u8], key_content: &str) -> Result<String> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(key_content.as_bytes());
    hasher.update(content);
    let result = hasher.finalize();
    Ok(format!("sig:sha256:{}", hex::encode(result)))
}
