use crate::chunker::{Chunk, Chunker};
use crate::types::ScraperConfig;
use crate::error::Result;
use crate::fetcher::Fetcher;
use crate::metadata::MetadataExtractor;
use crate::parser::{ContentParser, ParsedContent};
use crate::transformer::Transformer;
use crate::types::{ScraperInput, ScraperOutput, ScrapedChunk};
use chrono::Utc;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Main scraper orchestrator
#[allow(dead_code)]
pub struct Scraper {
    config: ScraperConfig,
    fetcher: Fetcher,
    parser: ContentParser,
    chunker: Chunker,
}

impl Scraper {
    /// Create a new scraper with configuration
    pub fn new(config: ScraperConfig) -> Result<Self> {
        let fetcher = Fetcher::new(config.clone())?;
        let parser = ContentParser::new(config.clone());
        let chunker = Chunker::new(config.clone());

        Ok(Self {
            config,
            fetcher,
            parser,
            chunker,
        })
    }

    /// Scrape from the given input and return chunks
    pub async fn scrape(&self, input: &ScraperInput) -> Result<ScraperOutput> {
        let start = Instant::now();
        let mut chunks = Vec::new();
        let mut file_count = 0;
        let mut total_bytes = 0u64;
        let mut errors = Vec::new();

        match input {
            ScraperInput::LocalPath(path) => {
                let content = self.fetcher.fetch_file(path).await?;
                total_bytes += content.len() as u64;
                file_count = 1;

                match self.process_file(path, &content, &mut chunks).await {
                    Ok(_) => {}
                    Err(e) => errors.push(e.to_string()),
                }
            }

            ScraperInput::Directory { path, patterns } => {
                let pattern_slice = patterns.as_ref().map(|p| p.as_slice());
                let files = self
                    .fetcher
                    .fetch_directory(path, pattern_slice)
                    .await?;

                file_count = files.len();

                for (file_path, content) in files {
                    total_bytes += content.len() as u64;
                    let full_path = path.join(&file_path);

                    match self.process_file(&full_path, &content, &mut chunks).await {
                        Ok(_) => {}
                        Err(e) => {
                            errors.push(format!("Error processing {}: {}", file_path.display(), e))
                        }
                    }
                }
            }

            ScraperInput::Url(url) => {
                let content = self.fetcher.fetch_url(url).await?;
                total_bytes += content.len() as u64;
                file_count = 1;

                let temp_path = std::path::Path::new(url);
                match self.process_file(temp_path, &content, &mut chunks).await {
                    Ok(_) => {}
                    Err(e) => errors.push(e.to_string()),
                }
            }

            ScraperInput::GitRepo { url, branch, commit } => {
                // For MVP, treat git repo similar to URL
                errors.push("Git repository scraping not yet implemented".to_string());
                tracing::warn!(
                    "Git scraping not implemented: {} branch={:?} commit={:?}",
                    url,
                    branch,
                    commit
                );
            }
        }

        let chunk_count = chunks.len();
        let scraped_chunks = self.convert_to_scraped_chunks(chunks)?;
        let manifest = self.create_manifest(&scraped_chunks)?;
        let duration_ms = start.elapsed().as_millis();

        tracing::info!(
            "Scraping complete: {} chunks from {} files in {}ms",
            chunk_count,
            file_count,
            duration_ms
        );

        Ok(ScraperOutput {
            chunk_count,
            file_count,
            total_bytes,
            chunks: scraped_chunks,
            manifest,
            errors,
            duration_ms,
        })
    }

    /// Process a single file
    async fn process_file(
        &self,
        file_path: &Path,
        content: &[u8],
        chunks: &mut Vec<(Chunk, ParsedContent, Option<serde_json::Value>)>,
    ) -> Result<()> {
        tracing::debug!("Processing file: {}", file_path.display());

        // Parse content
        let parsed = self.parser.parse(content, Some(file_path))?;

        // Extract metadata
        let _metadata = MetadataExtractor::extract(
            &parsed.text,
            Some(file_path),
        )?;

        // Try to parse as code for AST extraction
        let ast_info = if let Some(ref lang) = parsed.language {
            match self.parser.parse_code(&parsed.text, lang) {
                Ok(ast) => {
                    let features = Transformer::extract_features(&ast);
                    let quality = Transformer::compute_quality_metrics(&ast);
                    Some(json!({
                        "ast": {
                            "functions": ast.functions,
                            "classes": ast.classes,
                            "traits": ast.traits,
                            "imports": ast.imports,
                        },
                        "features": features,
                        "quality": {
                            "complexity": quality.cyclomatic_complexity_estimate,
                            "api_surface": quality.api_surface_size,
                            "dependencies": quality.dependency_count,
                            "modularity": quality.modularity_score,
                        }
                    }))
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // Chunk the content
        let file_chunks = self.chunker.chunk(
            &parsed.text,
            parsed.language.as_deref(),
            &file_path.to_string_lossy(),
        )?;

        for chunk in file_chunks {
            chunks.push((chunk, parsed.clone(), ast_info.clone()));
        }

        Ok(())
    }

    /// Convert internal chunks to ScrapedChunk format
    fn convert_to_scraped_chunks(
        &self,
        chunks: Vec<(Chunk, ParsedContent, Option<serde_json::Value>)>,
    ) -> Result<Vec<ScrapedChunk>> {
        let mut result = Vec::new();

        for (chunk, parsed, ast_info) in chunks {
            let mut concepts = chunk.concepts.clone();

            // Add parsed metadata concepts
            if let Some(ref meta) = parsed.metadata.title {
                concepts.push(format!("titled:{}", meta));
            }

            let mut dependencies = Vec::new();
            if let Some(ref ast) = ast_info {
                if let Some(imports) = ast.get("ast").and_then(|a| a.get("imports")) {
                    if let Some(imports_arr) = imports.as_array() {
                        for import in imports_arr {
                            if let Some(s) = import.as_str() {
                                dependencies.push(s.to_string());
                            }
                        }
                    }
                }
            }

            let name = parsed
                .metadata
                .title
                .clone()
                .unwrap_or_else(|| format!("chunk-{}", &chunk.id[..12]));

            result.push(ScrapedChunk {
                chunk_id: chunk.id,
                cadi_type: "source-cadi".to_string(),
                name,
                description: parsed.metadata.description.clone(),
                source: chunk.source_file.clone(),
                content_hash: compute_hash(&chunk.content),
                size: chunk.content.len(),
                language: parsed.language.clone(),
                concepts,
                dependencies,
                license: None,
                parent_chunk_id: chunk.parent_id.clone(),
                child_chunk_ids: chunk.children.clone(),
                tags: vec![],
                scraped_at: Utc::now().to_rfc3339(),
            });
        }

        Ok(result)
    }

    /// Create a manifest for all chunks
    fn create_manifest(&self, chunks: &[ScrapedChunk]) -> Result<Option<serde_json::Value>> {
        if chunks.is_empty() {
            return Ok(None);
        }

        let manifest = json!({
            "version": "1.0.0",
            "cadi_type": "manifest",
            "scraped_at": Utc::now().to_rfc3339(),
            "chunk_count": chunks.len(),
            "chunks": chunks.iter().map(|c| json!({
                "chunk_id": c.chunk_id,
                "name": c.name,
                "source": c.source,
                "language": c.language,
                "concepts": c.concepts,
            })).collect::<Vec<_>>(),
            "dependency_graph": self.build_dependency_graph(chunks)?,
        });

        Ok(Some(manifest))
    }

    /// Build a dependency graph from chunks
    fn build_dependency_graph(&self, chunks: &[ScrapedChunk]) -> Result<serde_json::Value> {
        let mut graph = HashMap::new();

        for chunk in chunks {
            let deps: Vec<String> = chunk
                .dependencies
                .iter()
                .map(|d| d.clone())
                .collect();

            graph.insert(chunk.chunk_id.clone(), deps);
        }

        Ok(serde_json::to_value(graph)?)
    }
}

/// Compute SHA256 hash
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scraper_creation() {
        let config = ScraperConfig::default();
        let scraper = Scraper::new(config);
        assert!(scraper.is_ok());
    }
}
