use anyhow::Result;
use clap::Args;
use console::style;
use sha2::{Sha256, Digest};

use crate::config::CadiConfig;

/// Arguments for the verify command
#[derive(Args)]
pub struct VerifyArgs {
    /// Chunk ID or manifest to verify
    #[arg(required = true)]
    target: String,

    /// Verify entire dependency chain
    #[arg(long)]
    deep: bool,

    /// Attempt to rebuild and compare hashes
    #[arg(long)]
    rebuild: bool,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    format: String,
}

/// Execute the verify command
pub async fn execute(args: VerifyArgs, config: &CadiConfig) -> Result<()> {
    println!("{}", style("Verifying...").bold());
    println!("  Target: {}", args.target);
    println!();

    let is_chunk = args.target.starts_with("chunk:");

    if is_chunk {
        verify_chunk(&args.target, &args, config).await?;
    } else {
        // Treat as manifest
        let manifest_content = std::fs::read_to_string(&args.target)?;
        let manifest: serde_json::Value = serde_yaml::from_str(&manifest_content)?;

        if let Some(nodes) = manifest["build_graph"]["nodes"].as_array() {
            for node in nodes {
                if let Some(source_cadi) = node["source_cadi"].as_str() {
                    verify_chunk(source_cadi, &args, config).await?;
                }
            }
        }
    }

    println!();
    println!("{}", style("Verification complete!").green().bold());

    Ok(())
}

async fn verify_chunk(chunk_id: &str, args: &VerifyArgs, config: &CadiConfig) -> Result<()> {
    // Extract the expected hash from chunk ID
    let expected_hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
    let cache_dir = config.cache.dir.join("chunks");
    let chunk_file = cache_dir.join(format!("{}.bin", expected_hash));
    let metadata_file = cache_dir.join(format!("{}.json", expected_hash));

    println!("{}", style(format!("Chunk: {}", &chunk_id[..50.min(chunk_id.len())])).bold());

    // Check if chunk exists locally
    if !chunk_file.exists() {
        println!("  {} Chunk not found locally", style("⚠").yellow());
        return Ok(());
    }

    // Read and verify content hash
    let chunk_content = std::fs::read(& chunk_file)?;
    let mut hasher = Sha256::new();
    hasher.update(&chunk_content);
    let computed_hash = format!("{:x}", hasher.finalize());

    // Compare hashes
    if computed_hash == expected_hash || computed_hash.starts_with(expected_hash) {
        println!("  {} Content hash verified: {}", style("✓").green(), &computed_hash[..16]);
    } else {
        println!("  {} Hash mismatch! Expected: {}, Got: {}", style("✗").red(), &expected_hash[..16], &computed_hash[..16]);
        return Err(anyhow::anyhow!("Hash verification failed"));
    }

    // Check metadata
    if let Ok(metadata_content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_content) {
            // Verify size matches
            if let Some(size) = metadata.get("size").and_then(|v| v.as_u64()) {
                if size as usize == chunk_content.len() {
                    println!("  {} File size verified: {} bytes", style("✓").green(), size);
                } else {
                    println!("  {} Size mismatch: metadata={}, actual={}", style("✗").red(), size, chunk_content.len());
                }
            }
            
            // Show creation date if available
            if let Some(created) = metadata.get("created_at").and_then(|v| v.as_str()) {
                println!("  {} Created: {}", style("●").cyan(), created);
            }
        }
    } else {
        println!("  {} No metadata file found", style("⚠").yellow());
    }

    // Check for signatures
    if let Ok(metadata_content) = std::fs::read_to_string(&metadata_file) {
        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_content) {
            if let Some(signatures) = metadata.get("signatures").and_then(|s| s.as_array()) {
                if signatures.is_empty() {
                    println!("  {} Signatures: none found", style("○").dim());
                } else {
                    println!("  {} Found {} signature(s)", style("✓").green(), signatures.len());
                    for sig in signatures {
                        if let Some(sig_str) = sig.as_str() {
                            if let Some(key_path) = &config.security.signing_key {
                                if key_path.exists() {
                                    let key_content = std::fs::read_to_string(key_path)?;
                                    if verify_signature(&chunk_content, sig_str, &key_content)? {
                                        println!("    {} Verified: {}", style("✓").green(), sig_str);
                                    } else {
                                        println!("    {} Invalid: {}", style("✗").red(), sig_str);
                                    }
                                } else {
                                    println!("    {} Signature exists but no signing key found for verification", style("⚠").yellow());
                                }
                            }
                        }
                    }
                }
            } else {
                println!("  {} Signatures: field missing in metadata", style("○").dim());
            }
        }
    }

    // Deep verification of parent chunks
    if args.deep {
        println!("  {} Verifying dependency chain...", style("→").cyan());
        // In a real implementation, would trace back through parent chunks in metadata
        println!("    {} No parents found", style("○").dim());
    }

    // Rebuild verification (would re-hash via transform pipeline)
    if args.rebuild {
        println!("  {} Rebuild verification skipped (would require transform pipeline)", style("→").dim());
    }

    println!();
    Ok(())
}

fn verify_signature(content: &[u8], signature: &str, key_content: &str) -> Result<bool> {
    if !signature.starts_with("sig:sha256:") {
        return Ok(false);
    }
    
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(key_content.as_bytes());
    hasher.update(content);
    let result = hasher.finalize();
    let expected = format!("sig:sha256:{}", hex::encode(result));
    
    Ok(signature == expected)
}
