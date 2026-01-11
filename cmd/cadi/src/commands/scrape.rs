use anyhow::{Context, Result};
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use cadi_scraper::{
    ScraperConfig, ScraperInput, ChunkingStrategy, Scraper,
};

use crate::config::CadiConfig;

/// Arguments for the scrape command
#[derive(Args)]
pub struct ScrapeArgs {
    /// Input path, URL, or directory to scrape
    #[arg(required = true)]
    input: String,

    /// Output directory for chunks
    #[arg(short, long, default_value = "./cadi-chunks")]
    output: PathBuf,

    /// Chunking strategy: file|semantic|fixed-size|hierarchical|by-line-count
    #[arg(short, long, default_value = "file")]
    strategy: String,

    /// Maximum chunk size in bytes
    #[arg(long, default_value = "52428800")] // 50MB
    max_chunk_size: usize,

    /// Include overlapping context between chunks
    #[arg(long, default_value = "true")]
    include_overlap: bool,

    /// Create hierarchical chunk relationships
    #[arg(long, default_value = "true")]
    hierarchy: bool,

    /// Extract API surfaces
    #[arg(long, default_value = "true")]
    extract_api: bool,

    /// Detect licenses
    #[arg(long, default_value = "true")]
    detect_licenses: bool,

    /// Publish to registry after scraping
    #[arg(long)]
    publish: bool,

    /// Registry namespace for publishing
    #[arg(long)]
    namespace: Option<String>,

    /// Output format: json|yaml|table
    #[arg(long, default_value = "table")]
    format: String,

    /// Dry run (don't save chunks)
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

/// Execute the scrape command
pub async fn execute(args: ScrapeArgs, _config: &CadiConfig) -> Result<()> {
    println!("{}", style("Starting CADI Scraper...").bold());

    // Parse input
    let input = parse_input(&args.input)?;
    println!("  {} Input: {}", style("→").cyan(), args.input);
    println!("  {} Output: {}", style("→").cyan(), args.output.display());

    // Create scraper configuration
    let mut config = ScraperConfig::default();
    config.max_chunk_size = args.max_chunk_size;
    config.include_overlap = args.include_overlap;
    config.create_hierarchy = args.hierarchy;
    config.extract_api_surface = args.extract_api;
    config.detect_licenses = args.detect_licenses;

    config.chunking_strategy = match args.strategy.as_str() {
        "semantic" => ChunkingStrategy::Semantic,
        "fixed-size" => ChunkingStrategy::FixedSize,
        "hierarchical" => ChunkingStrategy::Hierarchical,
        "by-line-count" => ChunkingStrategy::ByLineCount,
        _ => ChunkingStrategy::ByFile,
    };

    if args.verbose {
        println!("  {} Strategy: {:?}", style("→").cyan(), config.chunking_strategy);
        println!("  {} Max chunk size: {}", style("→").cyan(), args.max_chunk_size);
        println!("  {} Include overlap: {}", style("→").cyan(), args.include_overlap);
    }

    // Create scraper
    let scraper = Scraper::new(config).context("Failed to initialize scraper")?;

    // Show progress
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Scraping content...");

    // Execute scraping
    let output = scraper.scrape(&input).await.context("Scraping failed")?;
    spinner.finish_with_message(format!(
        "{} Scraped {} chunks from {} files ({} bytes)",
        style("✓").green(),
        output.chunk_count,
        output.file_count,
        output.total_bytes
    ));

    // Display results
    println!();
    println!("{}", style("Scraping Results:").bold());
    println!(
        "  {} Chunks: {}",
        style("→").cyan(),
        style(output.chunk_count).yellow()
    );
    println!(
        "  {} Files: {}",
        style("→").cyan(),
        style(output.file_count).yellow()
    );
    println!(
        "  {} Total Size: {} bytes",
        style("→").cyan(),
        output.total_bytes
    );
    println!(
        "  {} Duration: {}ms",
        style("→").cyan(),
        output.duration_ms
    );

    // Display errors if any
    if !output.errors.is_empty() {
        println!();
        println!("{}", style("Errors:").bold().red());
        for error in &output.errors {
            println!("  {} {}", style("✗").red(), error);
        }
    }

    // Save chunks if not dry-run
    if !args.dry_run {
        let spinner = ProgressBar::new(output.chunks.len() as u64);
        spinner.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{wide_bar:.cyan/blue} {pos}/{len}")
                .unwrap(),
        );
        spinner.set_message("Saving chunks...");

        // Create output directory
        tokio::fs::create_dir_all(&args.output)
            .await
            .context("Failed to create output directory")?;

        // Save each chunk
        for chunk in &output.chunks {
            let chunk_path = args.output.join(format!("{}.json", &chunk.chunk_id));
            let chunk_json = serde_json::to_string_pretty(chunk)?;
            tokio::fs::write(&chunk_path, chunk_json)
                .await
                .context(format!("Failed to write chunk {}", chunk.chunk_id))?;

            spinner.inc(1);
        }

        spinner.finish_with_message(format!(
            "{} Saved {} chunks to {}",
            style("✓").green(),
            output.chunks.len(),
            args.output.display()
        ));

        // Save manifest
        if let Some(manifest) = &output.manifest {
            let manifest_path = args.output.join("manifest.json");
            let manifest_json = serde_json::to_string_pretty(manifest)?;
            tokio::fs::write(&manifest_path, manifest_json).await?;
            println!(
                "  {} Manifest saved to {}",
                style("✓").green(),
                manifest_path.display()
            );
        }
    }

    // Display chunk details based on format
    match args.format.as_str() {
        "json" => {
            println!();
            println!("{}", style("Chunks (JSON):").bold());
            let json = serde_json::to_string_pretty(&output.chunks)?;
            println!("{}", json);
        }
        "table" => {
            println!();
            println!("{}", style("Chunk Details:").bold());
            for chunk in output.chunks.iter().take(5) {
                println!("  {} {}",  style("→").cyan(), chunk.name);
                println!("    {} Language: {:?}", style("·").blue(), chunk.language);
                println!("    {} Size: {} bytes", style("·").blue(), chunk.size);
                println!("    {} Concepts: {}", style("·").blue(), chunk.concepts.join(", "));
                if let Some(desc) = &chunk.description {
                    println!("    {} Description: {}", style("·").blue(), desc);
                }
            }
            if output.chunks.len() > 5 {
                println!(
                    "  {} ... and {} more chunks",
                    style("→").cyan(),
                    output.chunks.len() - 5
                );
            }
        }
        _ => {}
    }

    // Publish if requested
    if args.publish {
        println!();
        println!("{}", style("Publishing chunks...").bold());
        println!("  {} Registry publishing not yet configured", style("!").yellow());
    }

    println!();
    println!("{}", style("Scraping completed successfully!").green().bold());

    Ok(())
}

/// Parse input string into ScraperInput
fn parse_input(input: &str) -> Result<ScraperInput> {
    if input.starts_with("http://") || input.starts_with("https://") {
        Ok(ScraperInput::Url(input.to_string()))
    } else if input.starts_with("git@") || input.ends_with(".git") {
        Ok(ScraperInput::GitRepo {
            url: input.to_string(),
            branch: None,
            commit: None,
        })
    } else {
        let path = PathBuf::from(input);
        if path.is_dir() {
            Ok(ScraperInput::Directory {
                path,
                patterns: None,
            })
        } else if path.is_file() {
            Ok(ScraperInput::LocalPath(path))
        } else {
            Err(anyhow::anyhow!(
                "Input not found or not recognized: {}",
                input
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_url() {
        let input = parse_input("https://example.com/repo").unwrap();
        match input {
            ScraperInput::Url(url) => assert_eq!(url, "https://example.com/repo"),
            _ => panic!("Expected URL input"),
        }
    }

    #[test]
    fn test_parse_input_git() {
        let input = parse_input("git@github.com:user/repo.git").unwrap();
        match input {
            ScraperInput::GitRepo { .. } => {}
            _ => panic!("Expected GitRepo input"),
        }
    }
}
