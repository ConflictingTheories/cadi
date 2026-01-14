//! CADI Registry Database Layer
//!
//! Provides SurrealDB integration for:
//! - Vector search with MTREE indexing
//! - Chunk storage and retrieval
//! - Metadata normalization and indexing

use cadi_core::{CadiError, CadiResult, Chunk};
use cadi_llm::embeddings::{Embedding, EmbeddingManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Database schema for chunks with vector search
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub id: String,
    pub hash: String,
    pub content: String,
    pub language: String,
    pub metadata: ChunkMetadata,
    pub embedding: Option<Vec<f32>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

/// Normalized metadata for search indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub name: String,
    pub description: String,
    pub language: String,
    pub concepts: Vec<String>,
    pub dependencies: Vec<String>,
    pub function_signatures: Vec<String>,
    pub quality_score: f64,
    pub test_coverage: f64,
}

/// Search query structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub language: Option<String>,
    pub limit: usize,
    pub min_score: f64,
}

/// Search result from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSearchResult {
    pub chunk_id: String,
    pub score: f64,
    pub metadata: ChunkMetadata,
}

/// Registry database manager
pub struct RegistryDatabase {
    db: Surreal<Db>,
    embedding_manager: Option<EmbeddingManager>,
}

impl RegistryDatabase {
    /// Create a new registry database
    pub async fn new(db: Surreal<Db>, embedding_manager: Option<EmbeddingManager>) -> CadiResult<Self> {
        // Initialize schema
        Self::init_schema(&db).await?;

        Ok(Self { db, embedding_manager })
    }

    /// Initialize database schema
    async fn init_schema(db: &Surreal<Db>) -> CadiResult<()> {
        // Define chunk table with vector indexing
        let schema = r#"
            DEFINE TABLE chunk SCHEMAFULL;

            DEFINE FIELD id ON chunk TYPE string;
            DEFINE FIELD hash ON chunk TYPE string;
            DEFINE FIELD content ON chunk TYPE string;
            DEFINE FIELD language ON chunk TYPE string;
            DEFINE FIELD metadata ON chunk TYPE object;
            DEFINE FIELD embedding ON chunk TYPE array;
            DEFINE FIELD created_at ON chunk TYPE datetime;
            DEFINE FIELD usage_count ON chunk TYPE int DEFAULT 0;

            -- Vector index for semantic search
            DEFINE INDEX embedding_mtree ON chunk FIELDS embedding MTREE DIMENSION 384 DIST EUCLIDEAN;

            -- Text indexes for metadata search
            DEFINE INDEX metadata_name ON chunk FIELDS metadata.name;
            DEFINE INDEX metadata_concepts ON chunk FIELDS metadata.concepts;
            DEFINE INDEX metadata_language ON chunk FIELDS language;
        "#;

        db.query(schema).await.map_err(|e| CadiError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Store a chunk with metadata and embedding
    pub async fn store_chunk(&mut self, chunk: &Chunk, content: &str, metadata: ChunkMetadata) -> CadiResult<String> {
        let chunk_id = chunk.chunk_id.clone();
        let hash = chunk.chunk_id.strip_prefix("chunk:sha256:").unwrap_or(&chunk.chunk_id).to_string();

        // Generate embedding if manager available
        let embedding = if let Some(ref mut manager) = self.embedding_manager {
            // Create content for embedding from metadata
            let content_for_embedding = format!(
                "{} {} {}",
                metadata.name,
                metadata.description,
                metadata.concepts.join(" ")
            );
            match manager.get_chunk_embedding(&chunk_id, &content_for_embedding).await {
                Ok(embedding) => Some(embedding),
                Err(e) => return Err(CadiError::RegistryError(format!("Embedding generation failed: {}", e))),
            }
        } else {
            None
        };

        let record = ChunkRecord {
            id: chunk_id.clone(),
            hash,
            content: content.to_string(),
            language: "unknown".to_string(), // TODO: extract from chunk metadata
            metadata,
            embedding,
            created_at: chrono::Utc::now(),
            usage_count: 0,
        };

        // Store in database
        let sql = r#"
            CREATE chunk CONTENT $record
        "#;

        self.db.query(sql)
            .bind(("record", record))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        Ok(chunk_id)
    }

    /// Retrieve a chunk by ID
    pub async fn get_chunk(&self, chunk_id: &str) -> CadiResult<Option<ChunkRecord>> {
        let sql = r#"
            SELECT * FROM chunk WHERE id = $chunk_id
        "#;

        let mut response = self.db.query(sql)
            .bind(("chunk_id", chunk_id))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let results: Vec<ChunkRecord> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        Ok(results.into_iter().next())
    }

    /// Perform hybrid search
    pub async fn search(&self, query: SearchQuery) -> CadiResult<Vec<DbSearchResult>> {
        let mut results = Vec::new();

        // Text-based search
        if let Some(text) = &query.text {
            let text_results = self.text_search(text, query.limit * 2).await?;
            results.extend(text_results);
        }

        // Semantic search
        if let Some(embedding) = &query.embedding {
            let semantic_results = self.semantic_search(embedding, query.limit * 2).await?;
            results.extend(semantic_results);
        }

        // Combine and deduplicate results
        let mut combined = HashMap::new();
        for result in results {
            let entry = combined.entry(result.chunk_id.clone()).or_insert(result.clone());
            // Take the higher score if duplicate
            if result.score > entry.score {
                *entry = result;
            }
        }

        let mut final_results: Vec<_> = combined.into_values().collect();

        // Filter by minimum score and language
        final_results.retain(|r| r.score >= query.min_score);
        if let Some(lang) = &query.language {
            final_results.retain(|r| r.metadata.language == *lang);
        }

        // Sort by score and limit
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        final_results.truncate(query.limit);

        Ok(final_results)
    }

    /// Text-based search using fuzzy matching
    async fn text_search(&self, query: &str, limit: usize) -> CadiResult<Vec<DbSearchResult>> {
        let sql = r#"
            SELECT
                id,
                metadata,
                (string::similarity::fuzzy(metadata.name, $query) +
                 string::similarity::fuzzy(metadata.description, $query)) / 2.0 AS score
            FROM chunk
            WHERE score > 0.3
            ORDER BY score DESC
            LIMIT $limit
        "#;

        let mut response = self.db.query(sql)
            .bind(("query", query))
            .bind(("limit", limit))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let results: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let search_results = results.into_iter()
            .filter_map(|row| {
                let chunk_id = row.get("id")?.as_str()?.to_string();
                let score = row.get("score")?.as_f64()?;
                let metadata: ChunkMetadata = serde_json::from_value(row.get("metadata")?.clone()).ok()?;

                Some(DbSearchResult {
                    chunk_id,
                    score,
                    metadata,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Semantic search using vector similarity
    async fn semantic_search(&self, query_embedding: &[f32], limit: usize) -> CadiResult<Vec<DbSearchResult>> {
        let sql = r#"
            SELECT
                id,
                metadata,
                vector::similarity::cosine(embedding, $query_embedding) AS score
            FROM chunk
            WHERE embedding IS NOT NULL
            ORDER BY score DESC
            LIMIT $limit
        "#;

        let mut response = self.db.query(sql)
            .bind(("query_embedding", query_embedding))
            .bind(("limit", limit))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let results: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let search_results = results.into_iter()
            .filter_map(|row| {
                let chunk_id = row.get("id")?.as_str()?.to_string();
                let score = row.get("score")?.as_f64()?;
                let metadata: ChunkMetadata = serde_json::from_value(row.get("metadata")?.clone()).ok()?;

                Some(DbSearchResult {
                    chunk_id,
                    score,
                    metadata,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Increment usage count for a chunk
    pub async fn increment_usage(&self, chunk_id: &str) -> CadiResult<()> {
        let sql = r#"
            UPDATE chunk SET usage_count += 1 WHERE id = $chunk_id
        "#;

        self.db.query(sql)
            .bind(("chunk_id", chunk_id))
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get chunk statistics
    pub async fn get_stats(&self) -> CadiResult<RegistryStats> {
        let sql = r#"
            SELECT
                count() AS total_chunks,
                math::sum(usage_count) AS total_usage
            FROM chunk
        "#;

        let mut response = self.db.query(sql)
            .await
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        let stats: Vec<RegistryStats> = response.take(0)
            .map_err(|e| CadiError::DatabaseError(e.to_string()))?;

        Ok(stats.into_iter().next().unwrap_or_default())
    }
}

/// Registry statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_chunks: u64,
    pub total_usage: u64,
}