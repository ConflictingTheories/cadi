//! CADI Search Engine - Multi-modal search for semantic code discovery
//!
//! Supports hybrid search combining:
//! - Textual: BM25 scoring on metadata
//! - Semantic: Embedding similarity
//! - Structural: Function signature matching
//! - Compositional: Dependency graph analysis

use cadi_core::{Chunk, CadiResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Search modalities available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchModality {
    /// Text-based search (BM25 on metadata)
    Textual,
    /// Semantic embedding similarity
    Semantic,
    /// Structural signature matching
    Structural,
    /// Compositional dependency analysis
    Compositional,
    /// Hybrid combination of all modalities
    Hybrid,
}

/// Component metadata for search indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Programming language
    pub language: String,
    /// Usage count (popularity)
    pub usage_count: u64,
    /// Test coverage percentage (0.0-1.0)
    pub test_coverage: f64,
    /// Quality score (0.0-1.0)
    pub quality_score: f64,
    /// Key concepts/tags
    pub concepts: Vec<String>,
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Component ID
    pub id: String,
    /// Component metadata
    pub metadata: ComponentMetadata,
    /// Relevance score (0.0-1.0)
    pub score: f64,
    /// Which modality contributed most
    pub primary_modality: SearchModality,
}

/// Multi-modal search engine
pub struct SearchEngine {
    /// In-memory index of components
    index: HashMap<String, ComponentMetadata>,
    /// Embedding manager for semantic search
    embedding_manager: Option<cadi_llm::embeddings::EmbeddingManager>,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            embedding_manager: None,
        }
    }

    /// Create a new search engine with embedding support
    pub fn with_embeddings(embedding_manager: cadi_llm::embeddings::EmbeddingManager) -> Self {
        Self {
            index: HashMap::new(),
            embedding_manager: Some(embedding_manager),
        }
    }

    /// Register a chunk and extract metadata automatically
    pub fn register_chunk(&mut self, chunk: &Chunk, content: &str) -> CadiResult<String> {
        let chunk_id = chunk.chunk_id.clone();
        let metadata = MetadataNormalizer::normalize_chunk(chunk, content)?;
        self.register(&chunk_id, metadata);
        Ok(chunk_id)
    }

    /// Register a component with metadata
    pub fn register(&mut self, id: &str, metadata: ComponentMetadata) {
        self.index.insert(id.to_string(), metadata);
    }

    /// Search for components using hybrid search (async)
    pub async fn search(&mut self, query: &str, limit: usize) -> Vec<SearchResult> {
        // For now, just use text search. Semantic search can be added later
        // when we have proper async handling
        self.search_textual(query, limit)
    }

    /// Synchronous search for components (text-only for compatibility)
    pub fn search_sync(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for (id, metadata) in &self.index {
            let score = self.calculate_text_score(query, metadata);

            if score > 0.0 {
                results.push(SearchResult {
                    id: id.clone(),
                    metadata: metadata.clone(),
                    score,
                    primary_modality: SearchModality::Textual,
                });
            }
        }

        // Sort by score descending and limit results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);

        results
    }

    /// Perform textual search
    fn search_textual(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for (id, metadata) in &self.index {
            let score = self.calculate_text_score(query, metadata);

            if score > 0.0 {
                results.push(SearchResult {
                    id: id.clone(),
                    metadata: metadata.clone(),
                    score,
                    primary_modality: SearchModality::Textual,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    /// Perform semantic search
    async fn search_semantic(&mut self, manager: &mut cadi_llm::embeddings::EmbeddingManager, query: &str, limit: usize) -> Vec<(String, f32)> {
        // Generate embeddings for all indexed components
        let mut embeddings = HashMap::new();
        for (id, metadata) in &self.index {
            // Use metadata as content for embedding
            let content = format!("{} {} {}", metadata.name, metadata.description, metadata.concepts.join(" "));
            if let Ok(embedding) = manager.get_chunk_embedding(id, &content).await {
                embeddings.insert(id.clone(), embedding);
            }
        }

        // Perform semantic search
        if let Ok(results) = manager.search(query, &embeddings, limit).await {
            results
        } else {
            Vec::new()
        }
    }

    /// Calculate text-based relevance score
    fn calculate_text_score(&self, query: &str, metadata: &ComponentMetadata) -> f64 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        // Name match (highest weight)
        if metadata.name.to_lowercase().contains(&query_lower) {
            score += 0.4;
        }

        // Description match
        if metadata.description.to_lowercase().contains(&query_lower) {
            score += 0.3;
        }

        // Concept matches
        for concept in &metadata.concepts {
            if concept.to_lowercase().contains(&query_lower) {
                score += 0.2;
            }
        }

        // Language match
        if metadata.language.to_lowercase().contains(&query_lower) {
            score += 0.1;
        }

        // Boost by quality metrics
        score *= 0.5 + (metadata.quality_score * 0.5);
        score *= 0.8 + (metadata.test_coverage * 0.2);

        score.min(1.0)
    }

    /// Calculate hybrid score combining text and semantic
    fn calculate_hybrid_score(&self, text_score: f64, semantic_score: f32) -> f64 {
        // Semantic score is cosine similarity (-1 to 1), convert to 0-1
        let normalized_semantic = (semantic_score + 1.0) / 2.0;

        // Weighted combination: 50% semantic, 30% text, 20% quality boost
        0.5 * normalized_semantic as f64 + 0.3 * text_score + 0.2 * text_score
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata normalizer - extracts searchable metadata from code
pub struct MetadataNormalizer;

impl MetadataNormalizer {
    /// Normalize a chunk into searchable metadata
    pub fn normalize_chunk(chunk: &Chunk, content: &str) -> CadiResult<ComponentMetadata> {
        // Extract language from chunk metadata or default to unknown
        let language = chunk.meta.tags.iter()
            .find(|tag| tag.starts_with("lang:"))
            .map(|tag| tag.strip_prefix("lang:").unwrap_or("unknown"))
            .unwrap_or("unknown")
            .to_string();

        let extractor = CodeExtractor::new(&language);
        let extracted = extractor.extract(content)?;

        // Calculate quality score before moving any fields
        let quality_score = Self::calculate_quality_score(&extracted);

        // Extract values after calculating score
        let name = extracted.name.unwrap_or_else(|| "Unnamed Component".to_string());
        let description = extracted.description.unwrap_or_else(|| "No description available".to_string());
        let test_coverage = extracted.test_coverage.unwrap_or(0.0);
        let concepts = extracted.concepts.clone();

        Ok(ComponentMetadata {
            name,
            description,
            language,
            usage_count: 0,
            test_coverage,
            quality_score,
            concepts,
        })
    }

    /// Calculate quality score based on extracted features
    fn calculate_quality_score(extracted: &ExtractedCode) -> f64 {
        let mut score = 0.5; // Base score

        // Documentation quality
        if extracted.description.is_some() {
            score += 0.1;
        }

        // Test coverage
        if let Some(coverage) = extracted.test_coverage {
            score += coverage * 0.2;
        }

        // Function signatures (indicates well-structured code)
        score += (extracted.function_signatures.len() as f64 * 0.05).min(0.1);

        // Concepts/tags (indicates semantic richness)
        score += (extracted.concepts.len() as f64 * 0.02).min(0.1);

        score.min(1.0)
    }
}

/// Extracted information from code
#[derive(Debug, Default)]
struct ExtractedCode {
    name: Option<String>,
    description: Option<String>,
    concepts: Vec<String>,
    function_signatures: Vec<String>,
    test_coverage: Option<f64>,
}

/// Code feature extractor
struct CodeExtractor {
    language: String,
}

impl CodeExtractor {
    fn new(language: &str) -> Self {
        Self {
            language: language.to_lowercase(),
        }
    }

    fn extract(&self, code: &str) -> CadiResult<ExtractedCode> {
        let mut extracted = ExtractedCode::default();

        match self.language.as_str() {
            "typescript" | "javascript" => self.extract_js_ts(code, &mut extracted),
            "rust" => self.extract_rust(code, &mut extracted),
            "python" => self.extract_python(code, &mut extracted),
            _ => self.extract_generic(code, &mut extracted),
        }

        Ok(extracted)
    }

    fn extract_js_ts(&self, code: &str, extracted: &mut ExtractedCode) {
        let lines: Vec<&str> = code.lines().collect();

        // Extract top comment as description
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                let desc = trimmed.trim_start_matches("//").trim_start_matches("/*").trim();
                if !desc.is_empty() {
                    extracted.description = Some(desc.to_string());
                    break;
                }
            }
        }

        // Extract function signatures
        for line in &lines {
            if line.contains("function ") || line.contains("=>") || (line.contains("const ") && line.contains("=")) {
                extracted.function_signatures.push(line.trim().to_string());
            }
        }

        // Extract concepts from keywords
        let keywords = ["auth", "jwt", "http", "server", "database", "api", "middleware"];
        for keyword in &keywords {
            if code.to_lowercase().contains(keyword) {
                extracted.concepts.push(keyword.to_string());
            }
        }
    }

    fn extract_rust(&self, code: &str, extracted: &mut ExtractedCode) {
        let lines: Vec<&str> = code.lines().collect();

        // Extract doc comments
        for line in &lines {
            if line.trim().starts_with("///") {
                let desc = line.trim().trim_start_matches("///").trim();
                if !desc.is_empty() {
                    extracted.description = Some(desc.to_string());
                    break;
                }
            }
        }

        // Extract function signatures
        for line in &lines {
            if line.contains("fn ") {
                extracted.function_signatures.push(line.trim().to_string());
            }
        }

        // Extract concepts
        let keywords = ["async", "tokio", "serde", "http", "database", "api"];
        for keyword in &keywords {
            if code.to_lowercase().contains(keyword) {
                extracted.concepts.push(keyword.to_string());
            }
        }
    }

    fn extract_python(&self, code: &str, extracted: &mut ExtractedCode) {
        let lines: Vec<&str> = code.lines().collect();

        // Extract docstrings
        let mut in_docstring = false;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                in_docstring = !in_docstring;
                if in_docstring && extracted.description.is_none() {
                    extracted.description = Some(trimmed.trim_matches('\"').trim_matches('\'').to_string());
                }
            }
        }

        // Extract function signatures
        for line in &lines {
            if line.trim().starts_with("def ") {
                extracted.function_signatures.push(line.trim().to_string());
            }
        }

        // Extract concepts
        let keywords = ["flask", "django", "fastapi", "sqlalchemy", "requests", "api"];
        for keyword in &keywords {
            if code.to_lowercase().contains(keyword) {
                extracted.concepts.push(keyword.to_string());
            }
        }
    }

    fn extract_generic(&self, code: &str, extracted: &mut ExtractedCode) {
        // Basic extraction for unsupported languages
        let lines: Vec<&str> = code.lines().collect();

        // First comment line as description
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
                let desc = trimmed.trim_start_matches("//").trim_start_matches("#").trim_start_matches("/*").trim();
                if !desc.is_empty() {
                    extracted.description = Some(desc.to_string());
                    break;
                }
            }
        }

        // Extract concepts from common keywords
        let keywords = ["function", "class", "method", "api", "service", "handler"];
        for keyword in &keywords {
            if code.to_lowercase().contains(keyword) {
                extracted.concepts.push(keyword.to_string());
            }
        }
    }
}