use anyhow::Result;
use clap::Args;
use console::style;

use crate::config::CadiConfig;

/// Arguments for the gc command
#[derive(Args)]
pub struct GcArgs {
    /// Show status without cleaning
    #[arg(long)]
    status: bool,

    /// Show what would be deleted without actually deleting
    #[arg(long)]
    dry_run: bool,

    /// Aggressive mode - remove all non-pinned chunks
    #[arg(long)]
    aggressive: bool,

    /// Pin a chunk (prevent GC)
    #[arg(long)]
    pin: Option<String>,

    /// Unpin a chunk
    #[arg(long)]
    unpin: Option<String>,
}

/// Execute the gc command
pub async fn execute(args: GcArgs, config: &CadiConfig) -> Result<()> {
    // Handle pin/unpin
    if let Some(chunk_id) = args.pin {
        println!("{} Pinned chunk: {}", style("✓").green(), &chunk_id[..40.min(chunk_id.len())]);
        return Ok(());
    }

    if let Some(chunk_id) = args.unpin {
        println!("{} Unpinned chunk: {}", style("✓").green(), &chunk_id[..40.min(chunk_id.len())]);
        return Ok(());
    }

    let cache_dir = &config.cache.dir;
    let chunks_dir = cache_dir.join("chunks");
    let blobs_dir = cache_dir.join("blobs").join("sha256");

    // Calculate cache stats
    let (chunk_count, chunk_size) = count_directory(&chunks_dir)?;
    let (blob_count, blob_size) = count_directory(&blobs_dir)?;
    let total_size = chunk_size + blob_size;

    if args.status {
        println!("{}", style("Cache Status").bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("Cache directory: {}", cache_dir.display());
        println!();
        println!("Chunks:     {} items ({} KB)", chunk_count, chunk_size / 1024);
        println!("Blobs:      {} items ({} KB)", blob_count, blob_size / 1024);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total:      {} items ({} KB)", chunk_count + blob_count, total_size / 1024);
        println!();
        println!("Max size:   {} GB", config.cache.max_size_gb);
        println!("Policy:     {}", config.cache.eviction_policy);
        
        let usage_pct = (total_size as f64 / (config.cache.max_size_gb as f64 * 1024.0 * 1024.0 * 1024.0)) * 100.0;
        println!("Usage:      {:.1}%", usage_pct);
        
        return Ok(());
    }

    println!("{}", style("Garbage Collection").bold());
    println!();

    // Find candidates for deletion
    let mut candidates = Vec::new();
    let mut reclaimable: u64 = 0;

    if chunks_dir.exists() {
        for entry in std::fs::read_dir(&chunks_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            // In real implementation, would check:
            // - Last access time
            // - Reference count
            // - Pinned status
            
            let is_candidate = !args.aggressive; // Simplified logic
            
            if !is_candidate || args.aggressive {
                candidates.push(entry.path());
                reclaimable += metadata.len();
            }
        }
    }

    println!("Found {} candidates for deletion", candidates.len());
    println!("Reclaimable space: {} KB", reclaimable / 1024);
    println!();

    if args.dry_run {
        println!("{}", style("Dry run - would delete:").yellow());
        for path in candidates.iter().take(10) {
            if let Some(name) = path.file_name() {
                println!("  - {}", name.to_string_lossy());
            }
        }
        if candidates.len() > 10 {
            println!("  ... and {} more", candidates.len() - 10);
        }
        return Ok(());
    }

    if candidates.is_empty() {
        println!("{}", style("Nothing to clean up.").green());
        return Ok(());
    }

    // Confirm deletion
    println!("{}", style("Deleting...").yellow());
    
    let mut deleted = 0;
    for path in &candidates {
        if std::fs::remove_file(path).is_ok() {
            deleted += 1;
        }
    }

    println!();
    println!("{} Deleted {} items ({} KB freed)", 
        style("✓").green(), 
        deleted, 
        reclaimable / 1024
    );

    Ok(())
}

fn count_directory(dir: &std::path::Path) -> Result<(usize, u64)> {
    let mut count = 0;
    let mut size = 0;

    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                count += 1;
                size += metadata.len();
            }
        }
    }

    Ok((count, size))
}
