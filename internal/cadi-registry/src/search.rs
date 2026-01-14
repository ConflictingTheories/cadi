//! CADI Search Engine - Multi-modal search for semantic code discovery
//!
//! Supports hybrid search combining:
//! - Textual: BM25 scoring on metadata
//! - Semantic: Embedding similarity
//! - Structural: Function signature matching
//! - Compositional: Dependency graph analysis

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

    /// Register a component in the search index
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