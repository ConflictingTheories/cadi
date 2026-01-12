use anyhow::{Result, anyhow};
use clap::Args;
use console::style;
use std::path::PathBuf;
use cadi_core::{Manifest, chunk::SourceCadi};

/// Arguments for the scaffold command
#[derive(Args)]
pub struct ScaffoldArgs {
    /// Manifest file to scaffold from
    #[arg(required = true)]
    manifest: PathBuf,

    /// Output directory (defaults to manifest directory)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Force overwrite existing files
    #[arg(short, long)]
    force: bool,
}

/// Execute the scaffold command
pub async fn execute(args: ScaffoldArgs, _config: &crate::config::CadiConfig) -> Result<()> {
    let manifest_path = if args.manifest.is_absolute() {
        args.manifest.clone()
    } else {
        std::env::current_dir()?.join(&args.manifest)
    };

    if !manifest_path.exists() {
        return Err(anyhow!("Manifest not found: {}", manifest_path.display()));
    }

    let output_dir = args.output.unwrap_or_else(|| {
        manifest_path.parent().unwrap_or(PathBuf::from(".").as_path()).to_path_buf()
    });

    println!("{}", style(format!("Scaffolding project from: {}", manifest_path.display())).bold());
    println!("  Output directory: {}", output_dir.display());

    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_yaml::from_str(&content)?;

    // Generate files based on nodes
    for node in &manifest.build_graph.nodes {
        // Find source representation
        let source_repr = match node.representations.iter().find(|r| r.form == "source") {
            Some(r) => r,
            None => {
                println!("  {} Node {} has no source representation, skipping", style("⚠").yellow(), node.id);
                continue;
            }
        };

        let chunk_id = &source_repr.chunk;
        let chunk_hash = chunk_id.strip_prefix("chunk:sha256:").unwrap_or(chunk_id);
        
        let chunk_meta_path = _config.cache.dir.join("chunks").join(format!("{}.json", chunk_hash));
        if !chunk_meta_path.exists() {
            println!("  {} Chunk metadata not found for {}: {}", style("✗").red(), node.id, chunk_id);
            continue;
        }

        let chunk_json = std::fs::read_to_string(&chunk_meta_path)?;
        let source_cadi: SourceCadi = serde_json::from_str(&chunk_json)?;

        println!("  🏗 Realizing node {} from chunk {}", style(&node.id).cyan(), &chunk_id[..15]);

        for source_file in &source_cadi.source.files {
            let file_path = output_dir.join(&node.id).join(&source_file.path);
            
            // Create parent directories for the file if they don't exist
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            if file_path.exists() && !args.force {
                println!("    {} Skipping existing file: {}", style("⚠").yellow(), source_file.path);
                continue;
            }

            // Fetch blob
            let blob_hash = source_file.hash.strip_prefix("sha256:").unwrap_or(&source_file.hash);
            let blob_path = _config.cache.dir.join("blobs").join("sha256").join(blob_hash);

            if !blob_path.exists() {
                println!("    {} Blob not found for file {}: {}", style("✗").red(), source_file.path, blob_hash);
                continue;
            }

            let content = std::fs::read(&blob_path)?;
            std::fs::write(&file_path, content)?;
            println!("    {} Realized {}", style("✓").green(), source_file.path);
        }
    }

    println!();
    println!("{}", style("Scaffolding complete!").green().bold());
    
    Ok(())
}
