//! Seamless Import Command
//!
//! The `cadi import` command provides effortless project import:
//! - Automatically detects project type and structure
//! - Intelligently chunks code into reusable atomic pieces
//! - Creates human-readable aliases for easy reference
//! - Detects compositions (chunks made of other chunks)
//! - Handles any codebase - from simple scripts to complex monorepos
//! - Optionally publishes directly to a CADI registry

use anyhow::{Context, Result};
use clap::Args;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

use cadi_core::{
    AtomicChunk, ChunkGranularity,
    ImportResult, ProjectAnalyzer, ProjectAnalyzerConfig,
    SmartChunkerConfig,
};

use cadi_registry::{RegistryClient, RegistryConfig as ClientRegistryConfig};

use crate::config::CadiConfig;

/// Arguments for the import command
#[derive(Args)]
pub struct ImportArgs {
    /// Path to the project to import (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Project name override
    #[arg(short, long)]
    pub name: Option<String>,

    /// Output directory for chunk data
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Namespace for aliases (e.g., "my-org")
    #[arg(long)]
    pub namespace: Option<String>,

    /// Chunking strategy: auto|atomic|semantic|hierarchical
    #[arg(long, default_value = "auto")]
    pub strategy: String,

    /// Prefer atomic chunks (don't split files)
    #[arg(long)]
    pub atomic: bool,

    /// Include composition chunks
    #[arg(long, default_value = "true")]
    pub compositions: bool,

    /// Minimum lines for a function to be its own chunk
    #[arg(long, default_value = "10")]
    pub min_function_lines: usize,

    /// Maximum chunk lines before splitting
    #[arg(long, default_value = "500")]
    pub max_chunk_lines: usize,

    /// Don't publish to registry (local only)
    #[arg(long)]
    pub no_publish: bool,

    /// Publish chunks directly to registry
    #[arg(long)]
    pub publish: bool,

    /// Registry URL (default: from config or https://registry.cadi.dev)
    #[arg(long)]
    pub registry: Option<String>,

    /// Authentication token for the registry
    #[arg(long, env = "CADI_AUTH_TOKEN")]
    pub auth_token: Option<String>,

    /// Maximum concurrent uploads when publishing
    #[arg(long, default_value = "4")]
    pub concurrency: usize,

    /// Skip chunks that already exist in registry
    #[arg(long, default_value = "true")]
    pub skip_existing: bool,

    /// Dry run - show what would be imported
    #[arg(long)]
    pub dry_run: bool,

    /// Show detailed output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format: human|json|yaml
    #[arg(long, default_value = "human")]
    pub format: String,
}

/// Execute the import command
pub async fn execute(args: ImportArgs, config: &CadiConfig) -> Result<()> {
    let path = args.path.canonicalize()
        .context("Failed to resolve project path")?;

    // Show header
    if args.format == "human" {
        println!();
        println!("{}", style("╭─────────────────────────────────────────────────╮").cyan());
        println!("{}", style("│         CADI Seamless Import                    │").cyan().bold());
        println!("{}", style("╰─────────────────────────────────────────────────╯").cyan());
        println!();
    }

    // Build analyzer configuration
    let chunker_config = SmartChunkerConfig {
        min_function_lines: args.min_function_lines,
        min_file_lines_to_split: if args.atomic { usize::MAX } else { 50 },
        max_chunk_lines: args.max_chunk_lines,
        extract_utilities: true,
        extract_types: true,
        group_related: true,
        prefer_atomic: args.atomic,
        namespace: args.namespace.clone(),
    };

    let analyzer_config = ProjectAnalyzerConfig {
        chunker_config,
        detect_compositions: args.compositions,
        namespace: args.namespace.clone(),
        ..Default::default()
    };

    let analyzer = ProjectAnalyzer::new(analyzer_config);

    // Set up progress display
    let mp = MultiProgress::new();
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap();

    let analyze_spinner = mp.add(ProgressBar::new_spinner());
    analyze_spinner.set_style(spinner_style.clone());
    analyze_spinner.enable_steady_tick(Duration::from_millis(100));
    analyze_spinner.set_message(format!("Analyzing project: {}", path.display()));

    // Run the import
    let result = analyzer.import_project(&path)
        .context("Failed to import project")?;

    analyze_spinner.finish_with_message(format!(
        "{} Analysis complete",
        style("✓").green()
    ));

    // Show results based on format
    match args.format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "summary": &result.summary,
                "chunks": &result.chunks,
                "compositions": &result.compositions,
                "aliases": &result.alias_registry.aliases,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(());
        }
        "yaml" => {
            let output = serde_json::json!({
                "summary": &result.summary,
                "chunks": &result.chunks,
                "compositions": &result.compositions,
                "aliases": &result.alias_registry.aliases,
            });
            println!("{}", serde_yaml::to_string(&output)?);
            return Ok(());
        }
        _ => {}
    }

    // Human-readable output
    print_summary(&result, &path, args.verbose)?;

    // Save chunks if not dry run
    if !args.dry_run {
        let output_dir = args.output.clone().unwrap_or_else(|| {
            config.cache.dir.join("chunks")
        });

        let save_spinner = mp.add(ProgressBar::new_spinner());
        save_spinner.set_style(spinner_style.clone());
        save_spinner.enable_steady_tick(Duration::from_millis(100));
        save_spinner.set_message("Saving chunks...");

        save_chunks(&result, &output_dir, config)?;

        save_spinner.finish_with_message(format!(
            "{} Saved {} chunks to {}",
            style("✓").green(),
            result.chunks.len() + result.compositions.len(),
            output_dir.display()
        ));

        // Create manifest
        let manifest_path = path.join(format!(
            "{}.cadi.yaml",
            result.summary.project_name.to_lowercase().replace(' ', "-")
        ));
        create_manifest(&result, &manifest_path)?;

        println!();
        println!("{}", style("Import complete!").green().bold());
        println!();
        println!("  {} Manifest: {}", style("→").cyan(), manifest_path.display());
        println!("  {} Chunks:   {}", style("→").cyan(), output_dir.display());

        // Publish to registry if requested
        if args.publish && !args.no_publish {
            println!();
            let publish_result = publish_chunks(
                &result,
                &args,
                config,
                &mp,
                &spinner_style,
            ).await?;

            println!();
            println!("{}", style("Publish Summary:").bold());
            println!("  {} Published: {}", style("✓").green(), publish_result.published);
            if publish_result.skipped > 0 {
                println!("  {} Skipped:   {}", style("→").yellow(), publish_result.skipped);
            }
            if publish_result.failed > 0 {
                println!("  {} Failed:    {}", style("✗").red(), publish_result.failed);
            }
            println!("  {} Bytes:     {}", style("→").cyan(), format_size(publish_result.bytes_published));
        }
    } else {
        println!();
        println!("{}", style("Dry run - no files written").yellow());
    }

    // Next steps
    println!();
    println!("{}", style("Next steps:").bold());
    println!("  {} View chunks:    cadi query --local", style("1.").cyan());
    println!("  {} Build project:  cadi build", style("2.").cyan());
    if !args.no_publish && !args.publish {
        println!("  {} Publish:        cadi publish", style("3.").cyan());
    }
    println!();

    Ok(())
}

/// Print the import summary
fn print_summary(result: &ImportResult, path: &Path, verbose: bool) -> Result<()> {
    let summary = &result.summary;
    let analysis = &result.analysis;

    println!();
    println!("{}", style("Project Analysis").bold().underlined());
    println!();
    println!("  {} Project:     {}", style("→").cyan(), style(&summary.project_name).white().bold());
    println!("  {} Path:        {}", style("→").cyan(), path.display());
    println!("  {} Type:        {}", style("→").cyan(), style(&summary.project_type).yellow());
    println!("  {} Language:    {}", style("→").cyan(), &analysis.primary_language);
    println!("  {} Files:       {}", style("→").cyan(), summary.total_files);
    println!("  {} Lines:       {}", style("→").cyan(), format_number(summary.total_lines));
    println!("  {} Duration:    {}ms", style("→").cyan(), summary.duration_ms);
    println!();

    // Chunk summary
    println!("{}", style("Chunks Created").bold().underlined());
    println!();
    println!("  {} Atomic chunks:      {}", 
        style("→").cyan(), 
        style(summary.atomic_chunks).green().bold()
    );
    println!("  {} Composition chunks: {}", 
        style("→").cyan(), 
        style(summary.composition_chunks).green().bold()
    );
    println!("  {} Aliases created:    {}", 
        style("→").cyan(), 
        style(summary.aliases_created).green().bold()
    );
    println!("  {} Skipped files:      {}", 
        style("→").cyan(), 
        summary.skipped_files
    );
    println!();

    // Category breakdown
    if !summary.categories.is_empty() {
        println!("{}", style("By Category").bold().underlined());
        println!();
        for (category, count) in &summary.categories {
            let icon = get_category_icon(category);
            println!("  {} {}: {}", icon, category, count);
        }
        println!();
    }

    // Verbose: show all chunks and aliases
    if verbose {
        println!("{}", style("Chunks").bold().underlined());
        println!();

        for chunk in &result.chunks {
            print_chunk_summary(chunk);
        }

        if !result.compositions.is_empty() {
            println!();
            println!("{}", style("Compositions").bold().underlined());
            println!();

            for chunk in &result.compositions {
                print_chunk_summary(chunk);
            }
        }
    } else {
        // Show just a few example chunks
        println!("{}", style("Sample Chunks").bold().underlined());
        println!();

        for chunk in result.chunks.iter().take(5) {
            print_chunk_summary(chunk);
        }

        if result.chunks.len() > 5 {
            println!("  {} ... and {} more", 
                style("→").dim(), 
                result.chunks.len() - 5
            );
        }
        println!();
        println!("  {} Use --verbose to see all chunks", style("Tip:").dim());
    }

    Ok(())
}

/// Print a single chunk summary
fn print_chunk_summary(chunk: &AtomicChunk) {
    let alias = chunk.primary_alias()
        .map(|a| a.full_path())
        .unwrap_or_else(|| "—".to_string());

    let granularity_icon = match chunk.granularity {
        ChunkGranularity::Function => "ƒ",
        ChunkGranularity::Type => "T",
        ChunkGranularity::Module => "M",
        ChunkGranularity::Package => "P",
        ChunkGranularity::Project => "●",
    };

    let category_icon = chunk.categories.first()
        .map(|c| get_category_icon(&format!("{:?}", c)))
        .unwrap_or("  ");

    let size_str = format_size(chunk.size);
    let loc_str = format!("{}L", chunk.metrics.loc);

    let is_composition = !chunk.composition.is_atomic;

    if is_composition {
        let component_count = chunk.composition.composed_of.len();
        println!(
            "  {} {} {} {} ({} components)",
            style(granularity_icon).cyan(),
            category_icon,
            style(&alias).white().bold(),
            style(format!("[{}]", size_str)).dim(),
            component_count
        );
    } else {
        println!(
            "  {} {} {} {} {}",
            style(granularity_icon).cyan(),
            category_icon,
            style(&alias).white(),
            style(format!("[{}]", size_str)).dim(),
            style(format!("({})", loc_str)).dim()
        );
    }

    // Show provides if any
    if !chunk.provides.is_empty() {
        let provides_str = chunk.provides.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        let suffix = if chunk.provides.len() > 3 {
            format!(" +{}", chunk.provides.len() - 3)
        } else {
            String::new()
        };
        println!("      {} provides: {}{}", style("↳").dim(), provides_str, suffix);
    }
}

/// Save chunks to disk
fn save_chunks(result: &ImportResult, output_dir: &Path, _config: &CadiConfig) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // Save atomic chunks
    for chunk in &result.chunks {
        let chunk_file = output_dir.join(format!("{}.json", 
            chunk.chunk_id.trim_start_matches("chunk:sha256:").chars().take(16).collect::<String>()
        ));
        let json = serde_json::to_string_pretty(chunk)?;
        std::fs::write(&chunk_file, json)?;
    }

    // Save composition chunks
    for chunk in &result.compositions {
        let chunk_file = output_dir.join(format!("{}.json",
            chunk.chunk_id.trim_start_matches("chunk:sha256:").chars().take(16).collect::<String>()
        ));
        let json = serde_json::to_string_pretty(chunk)?;
        std::fs::write(&chunk_file, json)?;
    }

    // Save alias registry
    let registry_file = output_dir.join("aliases.json");
    let registry_json = serde_json::to_string_pretty(&result.alias_registry)?;
    std::fs::write(&registry_file, registry_json)?;

    // Save summary
    let summary_file = output_dir.join("import-summary.json");
    let summary_json = serde_json::to_string_pretty(&result.summary)?;
    std::fs::write(&summary_file, summary_json)?;

    Ok(())
}

/// Statistics from publishing
struct PublishStats {
    published: usize,
    skipped: usize,
    failed: usize,
    bytes_published: usize,
}

/// Publish chunks to a registry
async fn publish_chunks(
    result: &ImportResult,
    args: &ImportArgs,
    config: &CadiConfig,
    mp: &MultiProgress,
    _spinner_style: &ProgressStyle,
) -> Result<PublishStats> {
    let registry_url = args.registry.clone()
        .or_else(|| Some(config.registry.url.clone()))
        .unwrap_or_else(|| "https://registry.cadi.dev".to_string());

    println!();
    println!("{}", style("Publishing to Registry").bold().underlined());
    println!("  {} Registry: {}", style("→").cyan(), &registry_url);
    println!();

    let registry_config = ClientRegistryConfig {
        url: registry_url.clone(),
        token: args.auth_token.clone(),
        max_concurrent: args.concurrency,
        ..Default::default()
    };

    let client = RegistryClient::new(registry_config)
        .context("Failed to create registry client")?;

    // Check registry health
    let health = client.health().await;
    if let Ok(status) = &health {
        if !status.healthy {
            println!("{}", style("⚠ Registry may be unhealthy").yellow());
        }
    }

    let mut stats = PublishStats {
        published: 0,
        skipped: 0,
        failed: 0,
        bytes_published: 0,
    };

    // Combine all chunks
    let all_chunks: Vec<&AtomicChunk> = result.chunks.iter()
        .chain(result.compositions.iter())
        .collect();

    let total = all_chunks.len();
    let progress = mp.add(ProgressBar::new(total as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    for chunk in all_chunks {
        progress.set_message(format!("Publishing {}", short_id(&chunk.chunk_id)));

        // Check if exists (skip if configured)
        if args.skip_existing {
            match client.chunk_exists(&chunk.chunk_id).await {
                Ok(true) => {
                    stats.skipped += 1;
                    progress.inc(1);
                    continue;
                }
                Ok(false) => {}
                Err(_) => {} // Try to publish anyway
            }
        }

        // Serialize chunk to JSON
        let data = serde_json::to_vec(chunk)?;
        let size = data.len();

        // Publish
        match client.publish_chunk(&chunk.chunk_id, &data).await {
            Ok(_) => {
                stats.published += 1;
                stats.bytes_published += size;
            }
            Err(e) => {
                if args.verbose {
                    println!("{} {} {}", 
                        style("✗").red(), 
                        short_id(&chunk.chunk_id),
                        style(e.to_string()).dim()
                    );
                }
                stats.failed += 1;
            }
        }

        progress.inc(1);
    }

    progress.finish_with_message(format!(
        "{} Published {} chunks",
        style("✓").green(),
        stats.published
    ));

    Ok(stats)
}

/// Get short form of chunk ID
fn short_id(chunk_id: &str) -> String {
    chunk_id
        .trim_start_matches("chunk:sha256:")
        .chars()
        .take(12)
        .collect()
}

/// Create a CADI manifest file
fn create_manifest(result: &ImportResult, path: &Path) -> Result<()> {
    let manifest_id = format!("app:uuid:{}", uuid::Uuid::new_v4());

    // Build nodes from chunks
    let nodes: Vec<serde_json::Value> = result.chunks.iter().map(|chunk| {
        let alias = chunk.primary_alias()
            .map(|a| a.full_path())
            .unwrap_or_else(|| chunk.name.clone());

        serde_json::json!({
            "id": alias,
            "source_cadi": chunk.chunk_id,
            "alias": alias,
            "granularity": format!("{:?}", chunk.granularity).to_lowercase(),
            "categories": chunk.categories.iter().map(|c| format!("{:?}", c).to_lowercase()).collect::<Vec<_>>(),
            "representations": [{
                "form": "source",
                "chunk": chunk.chunk_id
            }],
            "selection_strategy": "prefer_source"
        })
    }).collect();

    // Build edges for compositions
    let mut edges = Vec::new();
    for comp in &result.compositions {
        let comp_alias = comp.primary_alias()
            .map(|a| a.full_path())
            .unwrap_or_else(|| comp.name.clone());

        for dep in &comp.composition.composed_of {
            let dep_alias = dep.alias.clone()
                .unwrap_or_else(|| dep.chunk_id.clone());

            edges.push(serde_json::json!({
                "from": comp_alias,
                "to": dep_alias,
                "type": "composition"
            }));
        }
    }

    let manifest = serde_json::json!({
        "manifest_id": manifest_id,
        "manifest_version": "1.0",
        "application": {
            "name": result.summary.project_name,
            "description": format!("{} - imported by CADI", result.summary.project_name),
            "version": "0.1.0"
        },
        "build_graph": {
            "nodes": nodes,
            "edges": edges
        },
        "aliases": result.alias_registry.aliases,
        "build_targets": [{
            "name": "default",
            "platform": "any",
            "nodes": nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n["id"],
                    "prefer": ["source"]
                })
            }).collect::<Vec<_>>()
        }]
    });

    let yaml = serde_yaml::to_string(&manifest)?;
    std::fs::write(path, yaml)?;

    Ok(())
}

/// Format a number with commas
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Format size in human-readable form
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Get an icon for a category
fn get_category_icon(category: &str) -> &'static str {
    match category.to_lowercase().as_str() {
        "logic" => "⚙️",
        "data" => "📦",
        "utility" => "🔧",
        "api" => "🌐",
        "config" => "⚙️",
        "test" => "🧪",
        "docs" => "📄",
        "build" => "🔨",
        "ui" => "🎨",
        "backend" => "🖥️",
        "database" => "💾",
        _ => "  ",
    }
}
