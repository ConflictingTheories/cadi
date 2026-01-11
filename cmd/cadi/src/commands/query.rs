use anyhow::{Result, anyhow};
use clap::Args;
use console::style;
use reqwest::Client;
use serde_json::json;

use crate::config::CadiConfig;

/// Arguments for the query command
#[derive(Args)]
pub struct QueryArgs {
    /// Name or partial chunk ID to search for
    #[arg(short, long)]
    name: Option<String>,

    /// Query by chunk ID
    #[arg(short, long)]
    chunk_id: Option<String>,

    /// Filter by language
    #[arg(short, long)]
    language: Option<String>,

    /// Query specific registry
    #[arg(short, long)]
    registry: Option<String>,

    /// Output format (json, table)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Limit results
    #[arg(long, default_value = "10")]
    limit: usize,
}

/// Execute the query command
pub async fn execute(args: QueryArgs, config: &CadiConfig) -> Result<()> {
    let registry = args.registry.as_ref()
        .unwrap_or(&config.registry.url);

    if args.format == "table" {
        println!("{}", style("Querying registry...").bold());
        println!("  Registry: {}", registry);
    }

    // Build query
    let client = Client::new();
    let mut url = format!("{}/v1/chunks", registry);
    let mut params = Vec::new();

    if let Some(ref name) = args.name {
        params.push(format!("name={}", urlencoding::encode(name)));
    }

    if let Some(ref chunk_id) = args.chunk_id {
        params.push(format!("chunk_id={}", urlencoding::encode(chunk_id)));
    }

    if let Some(ref language) = args.language {
        params.push(format!("language={}", urlencoding::encode(language)));
    }

    params.push(format!("limit={}", args.limit));

    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    // Execute query
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Registry query failed: {}",
            response.status()
        ));
    }

    let data: serde_json::Value = response.json().await?;

    // Parse results - handle both array and object formats
    let chunks = if data.is_array() {
        data.as_array().unwrap()
    } else {
        data["chunks"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid response format"))?
    };

    if args.format == "json" {
        // Output as JSON - wrap array in object for consistency
        let output = if data.is_array() {
            json!({ "chunks": chunks })
        } else {
            data
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Output as table
        println!();
        if chunks.is_empty() {
            println!("  {} No matching chunks found", style("!").yellow());
        } else {
            println!("  {} {} chunk(s) found:", style("✓").green(), chunks.len());
            println!();

            for chunk in chunks {
                let chunk_id = chunk["chunk_id"]
                    .as_str()
                    .unwrap_or("unknown");
                let size = chunk["size"]
                    .as_u64()
                    .unwrap_or(0);
                let content_type = chunk["content_type"]
                    .as_str()
                    .unwrap_or("unknown");

                let display_id = if chunk_id.len() > 60 {
                    format!("{}...{}", &chunk_id[..30], &chunk_id[chunk_id.len()-10..])
                } else {
                    chunk_id.to_string()
                };

                println!("  {} {}", style("•").cyan(), style(display_id).bold());
                println!("    Size: {} bytes", size);
                println!("    Type: {}", content_type);
                println!();
            }
        }
    }

    Ok(())
}
